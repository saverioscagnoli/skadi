use axum::{
    Router,
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::net::TcpListener;
use tokio::process::Command;
use tokio::sync::oneshot;
use tower_http::services::ServeDir;
use traccia::{debug, error, info};

pub async fn start_server(
    port: u16,
    root_dir: PathBuf,
    ready_tx: oneshot::Sender<()>,
) -> Result<(), Box<dyn Error>> {
    let app = Router::new()
        .route("/healthcheck", get(healthcheck))
        .route("/exec", post(exec))
        .fallback_service(ServeDir::new(&root_dir));

    debug!("Serving directory: {}", root_dir.display());

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;

    info!("Server running on http://localhost:{}", port);

    if ready_tx.send(()).is_err() {
        return Err("Failed to send ready signal.".into());
    }

    axum::serve(listener, app).await?;
    Ok(())
}

pub async fn healthcheck() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

#[derive(Deserialize)]
struct ExecRequest {
    command: String,
    args: Option<Vec<String>>,
}

#[derive(Serialize)]
struct ExecMessage {
    success: bool,
    stdout: String,
    stderr: String,
    exit_code: Option<i32>,
}

async fn exec(Json(payload): Json<ExecRequest>) -> impl IntoResponse {
    debug!(
        "Executing command: {} with args: {:?}",
        payload.command, payload.args
    );

    let mut cmd = Command::new(&payload.command);

    if let Some(args) = payload.args {
        cmd.args(&args);
    }

    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code();
            let success = output.status.success();

            info!(
                "Command executed with exit code: {:?}, success: {}",
                exit_code, success
            );

            let response = ExecMessage {
                success,
                stdout,
                stderr,
                exit_code,
            };

            Json(response)
        }

        Err(e) => {
            error!("Failed to execute command: {}", e);
            let response = ExecMessage {
                success: false,
                stdout: String::new(),
                stderr: e.to_string(),
                exit_code: None,
            };

            Json(response)
        }
    }
}
