use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;

use crate::error::SkadiError;

use traccia::{error, info};

pub struct LocalServer {
    listener: TcpListener,
    root: PathBuf,
}

impl LocalServer {
    pub async fn new(port: u16) -> Result<Self, SkadiError> {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| SkadiError::ServerError(e.to_string()))?;

        Ok(Self {
            listener,
            root: PathBuf::from("."), // Default to current directory
        })
    }

    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = root.into();
        self
    }

    pub async fn run(self, ready_tx: oneshot::Sender<()>) -> Result<(), SkadiError> {
        let port = self
            .listener
            .local_addr()
            .map_err(|e| SkadiError::ServerError(e.to_string()))?
            .port();

        info!("Server running on http://localhost:{}", port);

        // Send a signal that the server is ready
        if let Err(_) = ready_tx.send(()) {
            error!("Failed to send ready signal");
        }

        let root_dir = Arc::new(self.root);

        loop {
            match self.listener.accept().await {
                Ok((stream, _)) => {
                    let root_dir = Arc::clone(&root_dir);

                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_request(stream, &root_dir).await {
                            error!("Error handling request: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Error accepting connection: {}", e);
                }
            }
        }
    }

    async fn handle_request(
        mut stream: TcpStream,
        root_dir: &Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut buffer = [0; 1024];
        let n = stream.read(&mut buffer).await?;

        let request = String::from_utf8_lossy(&buffer[..n]);
        let request_line = request.lines().next().unwrap_or("");

        // Parse the requested path
        let path = if let Some(path) = request_line.split_whitespace().nth(1) {
            if path == "/" {
                "/index.html"
            } else {
                path
            }
        } else {
            "/index.html"
        };

        let file_path = root_dir.join(&path[1..]); // Remove leading '/'

        let (status, content_type, body) = if file_path.exists() && file_path.starts_with(root_dir)
        {
            let content = fs::read(&file_path).await.unwrap_or_default();
            let content_type = Self::get_content_type(&file_path);
            ("200 OK", content_type, content)
        } else {
            let content = b"404 Not Found".to_vec();
            ("404 NOT FOUND", "text/plain", content)
        };

        let response = format!(
            "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
            status,
            content_type,
            body.len()
        );

        stream.write_all(response.as_bytes()).await?;
        stream.write_all(&body).await?;
        stream.flush().await?;

        Ok(())
    }

    fn get_content_type(path: &Path) -> &'static str {
        match path.extension().and_then(|s| s.to_str()) {
            Some("html") => "text/html",
            Some("css") => "text/css",
            Some("js") => "application/javascript",
            Some("json") => "application/json",
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("svg") => "image/svg+xml",
            Some("woff") => "font/woff",
            Some("woff2") => "font/woff2",
            _ => "text/plain",
        }
    }
}
