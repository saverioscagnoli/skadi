use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::{
    fs,
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

#[derive(Debug, Clone)]
pub enum OutputMode {
    /// Ignore all output
    Ignore,
    /// Pipe output to stdout/stderr in real-time
    Pipe,
    /// Capture output and return it
    Capture,
    /// Inherit parent's stdout/stderr
    Inherit,
}

#[derive(Debug, Clone)]
pub struct SpawnOptions {
    pub cwd: Option<PathBuf>,
    pub stdout: OutputMode,
    pub stderr: OutputMode,
}

impl Default for SpawnOptions {
    fn default() -> Self {
        Self {
            cwd: None,
            stdout: OutputMode::Inherit,
            stderr: OutputMode::Inherit,
        }
    }
}

#[derive(Debug)]
pub struct SpawnResult {
    pub exit_code: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}

pub struct Io;

impl Io {
    pub async fn clean<P: AsRef<Path>>(path: P) -> tokio::io::Result<()> {
        if fs::metadata(&path).await.is_ok() {
            fs::remove_dir_all(&path).await?;
        }

        fs::create_dir_all(&path).await
    }

    pub async fn spawn<C: AsRef<str>>(
        command: C,
        args: &[&str],
        options: SpawnOptions,
    ) -> tokio::io::Result<SpawnResult> {
        let mut cmd = Command::new(command.as_ref());

        if let Some(cwd) = options.cwd {
            cmd.current_dir(cwd);
        }

        cmd.args(args);

        // Configure stdout
        match options.stdout {
            OutputMode::Ignore => {
                cmd.stdout(Stdio::null());
            }
            OutputMode::Pipe | OutputMode::Capture => {
                cmd.stdout(Stdio::piped());
            }
            OutputMode::Inherit => {
                cmd.stdout(Stdio::inherit());
            }
        }

        // Configure stderr
        match options.stderr {
            OutputMode::Ignore => {
                cmd.stderr(Stdio::null());
            }
            OutputMode::Pipe | OutputMode::Capture => {
                cmd.stderr(Stdio::piped());
            }
            OutputMode::Inherit => {
                cmd.stderr(Stdio::inherit());
            }
        }

        let mut child = cmd.spawn()?;

        let stdout_output = String::new();
        let stderr_output = String::new();

        let stdout_mode = options.stdout.clone();
        let stderr_mode = options.stderr.clone();

        // Handle stdout
        if matches!(stdout_mode, OutputMode::Pipe | OutputMode::Capture) {
            if let Some(stdout) = child.stdout.take() {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();

                let should_print = matches!(stdout_mode, OutputMode::Pipe);
                let mut stdout_output_clone = String::new();
                let stdout_mode_clone = stdout_mode.clone();

                tokio::spawn(async move {
                    while let Ok(Some(line)) = lines.next_line().await {
                        if should_print {
                            println!("{}", line);
                        }
                        if matches!(stdout_mode_clone, OutputMode::Capture) {
                            stdout_output_clone.push_str(&line);
                            stdout_output_clone.push('\n');
                        }
                    }
                });
            }
        }

        // Handle stderr
        if matches!(stderr_mode, OutputMode::Pipe | OutputMode::Capture) {
            if let Some(stderr) = child.stderr.take() {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();

                let should_print = matches!(stderr_mode, OutputMode::Pipe);
                let mut stderr_output_clone = String::new();
                let stderr_mode_clone = stderr_mode.clone();

                tokio::spawn(async move {
                    while let Ok(Some(line)) = lines.next_line().await {
                        if should_print {
                            eprintln!("{}", line);
                        }
                        if matches!(stderr_mode_clone, OutputMode::Capture) {
                            stderr_output_clone.push_str(&line);
                            stderr_output_clone.push('\n');
                        }
                    }
                });
            }
        }

        let exit_status = child.wait().await?;

        Ok(SpawnResult {
            exit_code: exit_status.code(),
            stdout: if matches!(stdout_mode, OutputMode::Capture) {
                Some(stdout_output)
            } else {
                None
            },
            stderr: if matches!(stderr_mode, OutputMode::Capture) {
                Some(stderr_output)
            } else {
                None
            },
        })
    }

    // Convenience methods for common use cases
    pub async fn spawn_silent<C: AsRef<str>>(
        command: C,
        args: &[&str],
        cwd: Option<&PathBuf>,
    ) -> tokio::io::Result<SpawnResult> {
        Self::spawn(
            command,
            args,
            SpawnOptions {
                stdout: OutputMode::Ignore,
                stderr: OutputMode::Ignore,
                cwd: cwd.map(|p| p.clone()),
            },
        )
        .await
    }

    pub async fn spawn_with_output<C: AsRef<str>>(
        command: C,
        args: &[&str],
        cwd: Option<&PathBuf>,
    ) -> tokio::io::Result<SpawnResult> {
        Self::spawn(
            command,
            args,
            SpawnOptions {
                stdout: OutputMode::Pipe,
                stderr: OutputMode::Pipe,
                cwd: cwd.map(|p| p.clone()),
            },
        )
        .await
    }

    pub async fn spawn_and_capture<C: AsRef<str>>(
        command: C,
        args: &[&str],
        cwd: Option<&PathBuf>,
    ) -> tokio::io::Result<SpawnResult> {
        Self::spawn(
            command,
            args,
            SpawnOptions {
                stdout: OutputMode::Capture,
                stderr: OutputMode::Capture,
                cwd: cwd.map(|p| p.clone()),
            },
        )
        .await
    }
}
