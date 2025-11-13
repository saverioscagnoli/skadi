use axum::{
    Router,
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use common::util;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::{process::Command, sync::mpsc::UnboundedSender};
use tower_http::{
    cors::{self, CorsLayer},
    services::ServeDir,
};
use traccia::{debug, error, fatal, info};

#[derive(Debug)]
pub struct EventRequest {
    pub widget_label: String,
    pub event_name: String,
    pub payload: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListenBody {
    pub script: String,
    pub widget_label: String,
}

pub async fn start_server(
    port: u16,
    root_dir: PathBuf,
    ready_tx: oneshot::Sender<()>,
    event_tx: UnboundedSender<EventRequest>,
) -> Result<(), Box<dyn Error>> {
    let cors = CorsLayer::new().allow_origin(cors::Any);

    let mut app = Router::new()
        .route("/healthcheck", get(healthcheck))
        .route("/exec", post(exec))
        .route("/listen", post(listen))
        .layer(cors)
        .with_state(event_tx);

    if !util::dev() {
        app = Router::new()
            .nest("/backend", app)
            .fallback_service(ServeDir::new(&root_dir));
    }

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

pub async fn listen(
    State(event_tx): State<UnboundedSender<EventRequest>>,
    Json(body): Json<ListenBody>,
) {
    debug!(
        "Setting up listener for widget: {} with script: {}",
        body.widget_label, body.script
    );

    let widget_label = body.widget_label.clone();
    let event_name = body.script.clone();

    tokio::spawn(async move {
        let mut child = match Command::new("bash")
            .arg("-c")
            .arg(&body.script)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                error!("Failed to spawn command: {}", e);
                return;
            }
        };

        let stdout = child.stdout.take().expect("Failed to capture stdout");
        let mut reader = BufReader::new(stdout).lines();

        while let Ok(Some(line)) = reader.next_line().await {
            let event = EventRequest {
                widget_label: widget_label.clone(),
                event_name: event_name.clone(), // Clone here
                payload: line,
            };

            if event_tx.send(event).is_err() {
                error!("Failed to send event, receiver dropped");
                break;
            }
        }

        match child.wait().await {
            Ok(status) => debug!("Command exited with: {}", status),
            Err(e) => error!("Error waiting for command: {}", e),
        }
    });
}
