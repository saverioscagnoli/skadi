use crate::window::{WindowAction, WindowActionRequest};
use axum::{
    Router,
    extract::{ConnectInfo, Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use fastwebsockets::{FragmentCollector, Frame, OpCode, Payload, WebSocketError, upgrade};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::process::Stdio;
use std::sync::Arc;
use std::{collections::HashMap, error::Error};
use std::{collections::HashSet, path::PathBuf};
use tokio::sync::{mpsc, oneshot};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    sync::Mutex,
};

use tokio::{net::TcpListener, sync::RwLock};
use tokio::{process::Command, sync::mpsc::UnboundedSender};
use tower_http::{
    cors::{self, CorsLayer},
    services::ServeDir,
};
use traccia::{debug, error, info, warn};

#[derive(Debug, Clone, Deserialize)]
pub struct WindowActionHandlerBody {
    /// The label of the widget
    pub target_label: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommandOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExecRequest {
    command: String,
    args: Vec<String>,
    widget_label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebsocketStreamMessage {
    pub stream_id: String,
    pub data: String,
}

#[derive(Clone)]
pub struct AppState {
    pub window_tx: UnboundedSender<WindowActionRequest>,
    pub websocket_senders: Arc<RwLock<HashMap<String, UnboundedSender<WebsocketStreamMessage>>>>,
    /// The hash set consists of [widget_label+command_name+command_args]
    /// Concatenate them so we don't have to use a HashMap<String, Vec<String>>
    pub active_commands: Arc<Mutex<HashSet<String>>>,
}

pub async fn start_server(
    port: u16,
    root_dir: PathBuf,
    ready_tx: oneshot::Sender<()>,
    window_tx: UnboundedSender<WindowActionRequest>,
) -> Result<(), Box<dyn Error>> {
    let cors = CorsLayer::new().allow_origin(cors::Any);

    let app_state = AppState {
        window_tx,
        websocket_senders: Arc::new(RwLock::new(HashMap::new())),
        active_commands: Arc::new(Mutex::new(HashSet::new())),
    };

    let app = Router::new()
        .route("/healthcheck", get(healthcheck))
        .route("/exec", post(exec))
        .route("/ws", get(websocket_handler))
        .layer(cors)
        .with_state(app_state)
        .fallback_service(ServeDir::new(&root_dir));

    debug!("Serving directory: {}", root_dir.display());

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;

    info!("Server running on http://localhost:{}", port);

    if ready_tx.send(()).is_err() {
        return Err("Failed to send ready signal.".into());
    }

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}

pub async fn healthcheck() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

async fn exec(Json(body): Json<ExecRequest>) -> impl IntoResponse {
    info!(
        "Widget '{}' requested execution: {} {:?}",
        &body.widget_label, &body.command, &body.args
    );

    let output = Command::new(&body.command)
        .args(&body.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    match output {
        Ok(output) => {
            let success = output.status.success();
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            Json(CommandOutput {
                success,
                stdout,
                stderr,
            })
        }

        Err(e) => {
            warn!(
                "Widget '{}' execution request {} {:?} failed: {}",
                &body.widget_label, &body.command, &body.args, e
            );

            Json(CommandOutput {
                success: false,
                stdout: String::new(),
                stderr: e.to_string(),
            })
        }
    }
}

async fn websocket_handler(
    ws: upgrade::IncomingUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(app_state): State<AppState>,
) -> impl IntoResponse {
    debug!("WebSocket connection from: {}", addr);
    let (response, fut) = ws.upgrade().unwrap();

    tokio::spawn(async move {
        if let Err(e) = client_handler(fut, app_state).await {
            error!("Error handling websocket from {}: {}", addr, e);
        } else {
            debug!("WebSocket connection closed: {}", addr);
        }
    });

    response
}

async fn client_handler(
    fut: upgrade::UpgradeFut,
    app_state: AppState,
) -> Result<(), WebSocketError> {
    let mut ws = FragmentCollector::new(fut.await?);
    let (tx, mut rx) = mpsc::unbounded_channel::<WebsocketStreamMessage>();
    let mut label: Option<String> = None;
    let mut spawned_tasks: Vec<tokio::task::JoinHandle<()>> = Vec::new();

    let result = loop {
        tokio::select! {
            Some(message) = rx.recv() => {
                let stringified = serde_json::to_string(&message)
                    .expect("If this fails I'll kill myself");
                let payload = Payload::Borrowed(stringified.as_bytes());

                if let Err(e) = ws.write_frame(Frame::text(payload)).await {
                    error!("Failed to send via socket: {}", e);
                    break Err(e.into());
                }
            }

            frame_result = ws.read_frame() => {
                let frame = match frame_result {
                    Ok(f) => f,
                    Err(e) => break Err(e),
                };

                match frame.opcode {
                    OpCode::Close => break Ok(()),
                    OpCode::Text => {
                        let text = match String::from_utf8(frame.payload.to_vec()) {
                            Ok(t) => t,
                            Err(e) => {
                                error!("Invalid UTF-8 in message: {}", e);
                                continue;
                            }
                        };
                        let parts = text.split_whitespace().collect::<Vec<_>>();

                        if parts.is_empty() {
                            error!("Received an empty message");
                            continue;
                        }

                        match parts[0] {
                            // The identification is only required
                            // for sending messages from a socket to another.
                            // For example, when a user wants to open/close a widget from another widget.
                            "IDENTIFY" => {
                                let Some(new_label) = parts.get(1) else {
                                    error!("Trying to identify without a label");
                                    continue;
                                };

                                if let Some(old_label) = label.take() {
                                    let mut lock = app_state.websocket_senders.write().await;
                                    lock.remove(&old_label);
                                }

                                let mut lock = app_state.websocket_senders.write().await;
                                lock.insert(new_label.to_string(), tx.clone());
                                label = Some(new_label.to_string());
                                debug!("Identified client with label: {}", new_label);
                            }

                            "EXECUTE" => {
                                let Some(command) = parts.get(1) else {
                                    error!("Received execute signal without a command");
                                    continue;
                                };

                                let args = parts.iter().skip(2).copied().collect::<Vec<_>>();

                                debug!("Executing command: {} {:?}", command, args);
                                let output = match Command::new(command)
                                    .args(&args)
                                    .stdout(Stdio::piped())
                                    .stderr(Stdio::piped())
                                    .output()
                                    .await
                                {
                                    Ok(c) => c,
                                    Err(e) => {
                                        error!("Failed to execute '{} {:?}': {}", command, args, e);
                                        continue;
                                    }
                                };

                                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                                if !output.status.success() {
                                    error!("Command '{}' failed with status code {}", command, output.status.code().unwrap_or(-1));
                                }
                            }

                            "LISTEN" => {
                                let Some(stream_id) = parts.get(1) else {
                                    error!("Received listen signal without a stream ID");
                                    continue;
                                };
                                let stream_id = stream_id.to_string();

                                let Some(command) = parts.get(2) else {
                                    error!("Received listen signal without a command");
                                    continue;
                                };

                                let args = parts.iter().skip(3).copied().collect::<Vec<_>>();

                                debug!("{} is listening to {} {:?}", stream_id, command, args);

                                let mut child = match Command::new(command)
                                    .args(&args)
                                    .stdout(Stdio::piped())
                                    .stderr(Stdio::piped())
                                    .spawn()
                                {
                                    Ok(c) => c,
                                    Err(e) => {
                                        error!("Failed to execute '{} {:?}': {}", command, args, e);
                                        continue;
                                    }
                                };

                                let Some(stdout) = child.stdout.take() else {
                                    error!("Failed to capture stdout for '{} {:?}'", command, args);
                                    continue;
                                };

                                let tx_clone = tx.clone();
                                let handle = tokio::spawn(async move {
                                    let mut stdout_reader = BufReader::new(stdout).lines();

                                    loop {
                                        tokio::select! {
                                            Ok(Some(line)) = stdout_reader.next_line() => {
                                                let message = WebsocketStreamMessage {
                                                    stream_id: stream_id.to_string(),
                                                    data: line,
                                                };

                                                if tx_clone.send(message).is_err() {
                                                    break;
                                                }
                                            }
                                            else => break,
                                        }
                                    }

                                    let _ = child.wait().await;
                                });

                                spawned_tasks.push(handle);
                            }

                            p => {
                                error!("Received unknown protocol part: {}", p);
                            }
                        }
                    }

                    _ => {}
                }
            }
        }
    };

    if let Some(label) = label {
        let mut lock = app_state.websocket_senders.write().await;
        lock.remove(&label);
    }

    for handle in spawned_tasks {
        handle.abort();
    }

    result
}

pub async fn window_action_handler(
    State(app_state): State<AppState>,
    Path(payload): Path<String>,
    Json(body): Json<WindowActionHandlerBody>,
) -> impl IntoResponse {
    let action = match payload.as_str() {
        "show" => WindowAction::Show,
        "hide" => WindowAction::Hide,
        _ => {
            warn!("Received invalid action {}", &payload);
            return StatusCode::BAD_REQUEST;
        }
    };

    let request = WindowActionRequest {
        target: body.target_label,
        action,
    };

    if let Err(e) = app_state.window_tx.send(request) {
        error!("Failed to handle window action: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}
