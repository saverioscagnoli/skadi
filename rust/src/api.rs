use crate::{error::SkadiError, events::EventEmitter};
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use std::{path::PathBuf, process::Stdio};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    sync::oneshot,
};
use tower_http::services::ServeDir;
use traccia::{error, info};

use axum::{
    extract::{ws::WebSocket, ws::WebSocketUpgrade},
    response::Response,
};
use futures_util::StreamExt;
use serde_json::Value;

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(event_emitter): State<EventEmitter>,
) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, event_emitter))
}

async fn handle_websocket(socket: WebSocket, event_emitter: EventEmitter) {
    let (mut sender, mut receiver) = socket.split();

    while let Some(msg) = receiver.next().await {
        let Ok(msg) = msg else {
            continue;
        };
        let Ok(text) = msg.to_text() else {
            continue;
        };
        let Ok(data) = serde_json::from_str::<Value>(text) else {
            continue;
        };

        let Some("exec") = data.get("type").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(command) = data.get("path").and_then(|v| v.as_str()) else {
            continue;
        };

        // Spawn command execution in background to avoid blocking
        let command = command.to_string();
        tokio::spawn(async move {
            let _ = Command::new("sh").arg("-c").arg(&command).spawn();
        });
    }
}

pub async fn asset_server(
    port: u16,
    root_dir: PathBuf,
    ready_tx: oneshot::Sender<()>,
    event_emitter: EventEmitter,
) -> Result<(), SkadiError> {
    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .route("/healthcheck", get(healthcheck))
        .fallback_service(ServeDir::new(root_dir))
        .with_state(event_emitter);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await?;

    info!("Server running on http://localhost:{}", port);

    if let Err(_) = ready_tx.send(()) {
        error!("Failed to send ready signal");
        return Err(SkadiError::BackendError(
            "Failed to send ready signal".into(),
        ));
    }

    axum::serve(listener, app).await?;
    Ok(())
}

async fn healthcheck() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

#[derive(Deserialize)]
struct ExecBody {
    command: String,
    polls: bool,
}

async fn exec(
    State(event_emitter): State<EventEmitter>,
    headers: HeaderMap,
    Json(body): Json<ExecBody>,
) -> impl IntoResponse {
    let command_path = PathBuf::from(&body.command);

    if body.polls {
        let window_label = headers.get("x-window-label").and_then(|h| h.to_str().ok());
        // Handle polling/streaming commands
        let mut child: Child;

        if command_path.exists() {
            child = match Command::new(&body.command)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(child) => child,
                Err(e) => {
                    let error_response = serde_json::json!({
                        "error": format!("Failed to spawn command: {}", e)
                    });

                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
                        .into_response();
                }
            };
        } else {
            child = match Command::new("sh")
                .arg("-c")
                .arg(&body.command)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(child) => child,
                Err(e) => {
                    let error_response = serde_json::json!({
                        "error": format!("Failed to spawn command: {}", e)
                    });
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
                        .into_response();
                }
            };
        }

        let stdout = child.stdout.take().unwrap();
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        let name = command_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown-command")
            .to_string();

        let window_label = window_label.unwrap_or("unknown-label").to_string();
        let event_emitter_clone = event_emitter.clone();

        // Spawn a task to handle streaming output
        tokio::spawn(async move {
            while let Ok(Some(line)) = lines.next_line().await {
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&line) {
                    event_emitter_clone
                        .emit_to(&window_label, &name, json_value)
                        .await;
                } else {
                    // If it's not valid JSON, emit as plain text
                    event_emitter_clone
                        .emit_to(&window_label, &name, serde_json::json!({ "output": line }))
                        .await;
                }
            }
        });

        (
            StatusCode::OK,
            Json(serde_json::json!({ "message": "Polling command started" })),
        )
            .into_response()
    } else {
        // Handle one-time commands (existing logic)
        let output;

        if command_path.exists() {
            output = Command::new(&body.command).output().await;
        } else {
            output = Command::new("sh")
                .arg("-c")
                .arg(&body.command)
                .output()
                .await;
        }

        match output {
            Ok(output) => (
                StatusCode::OK,
                Json(String::from_utf8_lossy(&output.stdout)),
            )
                .into_response(),

            Err(e) => {
                let error_response = serde_json::json!({
                    "error": format!("Failed to execute command: {}", e)
                });

                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
            }
        }
    }
}
