use std::{
    process::Stdio,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
    sync::Mutex,
    time::timeout,
};
use traccia::warn;

#[derive(Debug)]
pub struct BashProcess {
    stdin: ChildStdin,
    stdout_reader: Arc<Mutex<BufReader<ChildStdout>>>,
    child: Child,
}

impl BashProcess {
    pub async fn new() -> tokio::io::Result<Self> {
        let mut child = Command::new("bash")
            .arg("-c")
            .arg("exec -a '[wwwidgets-bash-pool]' bash -c 'while read -r cmd; do eval \"$cmd\"; echo \"<<<END_OF_OUTPUT>>>\"; done'")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let stdout_reader = Arc::new(Mutex::new(BufReader::new(stdout)));

        Ok(Self {
            stdin,
            stdout_reader,
            child,
        })
    }

    async fn ensure_alive(&mut self) -> tokio::io::Result<()> {
        match self.child.try_wait()? {
            Some(status) => {
                warn!("Bash process died with status: {:?}, recreating...", status);
                let new_instance = Self::new().await?;
                self.stdin = new_instance.stdin;
                self.stdout_reader = new_instance.stdout_reader;
                self.child = new_instance.child;

                Ok(())
            }
            None => Ok(()), // Still running
        }
    }

    pub async fn execute<C: AsRef<str>>(
        &mut self,
        command: C,
        args: Option<Vec<String>>,
    ) -> tokio::io::Result<String> {
        self.ensure_alive().await?;

        let command = command.as_ref();
        let write_result = self.stdin.write_all(command.as_bytes()).await;

        if write_result.is_err() {
            self.ensure_alive().await?;
            self.stdin.write_all(command.as_bytes()).await?;
        } else {
            write_result?;
        }

        if let Some(args) = args {
            for arg in args {
                self.stdin.write_all(b" ").await?;
                self.stdin.write_all(arg.as_bytes()).await?;
            }
        }
        self.stdin.write_all(b"\n").await?;
        self.stdin.flush().await?;

        let read_future = async {
            let mut output = String::new();
            let mut reader = self.stdout_reader.lock().await;
            let mut line = String::new();

            loop {
                let bytes_read = reader.read_line(&mut line).await?;

                if bytes_read == 0 {
                    break;
                }

                if line.trim() == "<<<END_OF_OUTPUT>>>" {
                    break;
                }

                output.push_str(&line);
                line.clear();
            }

            Ok(output)
        };

        match timeout(Duration::from_secs(30), read_future).await {
            Ok(result) => result,
            Err(_) => Err(tokio::io::Error::new(
                tokio::io::ErrorKind::TimedOut,
                "Command execution timed out after 30 seconds",
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BashPool {
    processes: Vec<Arc<Mutex<BashProcess>>>,
    current_index: Arc<AtomicUsize>,
    pool_size: usize,
}

impl BashPool {
    pub async fn new(pool_size: usize) -> tokio::io::Result<Self> {
        let mut processes = Vec::new();

        for _ in 0..pool_size {
            let process = BashProcess::new().await?;
            processes.push(Arc::new(Mutex::new(process)));
        }

        Ok(Self {
            processes,
            current_index: Arc::new(AtomicUsize::new(0)),
            pool_size,
        })
    }

    pub async fn execute<C: AsRef<str>>(
        &self,
        command: C,
        args: Option<Vec<String>>,
    ) -> tokio::io::Result<String> {
        let index = self.current_index.fetch_add(1, Ordering::Relaxed) % self.pool_size;

        let process = Arc::clone(&self.processes[index]);
        let mut process = process.lock().await;

        process.execute(command, args).await
    }
}
