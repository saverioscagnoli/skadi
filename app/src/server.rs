use crate::{
    bash::{BashPool, BashProcess},
    window::{WindowAction, WindowActionRequest},
};
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
#[serde(rename_all = "camelCase")]
struct ExecRequest {
    command: String,
    args: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebsocketStreamMessage {
    pub stream_id: Arc<String>,
    pub data: String,
}

#[derive(Clone)]
pub struct AppState {
    pub window_tx: UnboundedSender<WindowActionRequest>,
    pub websocket_senders: Arc<RwLock<HashMap<String, UnboundedSender<WebsocketStreamMessage>>>>,
    /// The hash set consists of [widget_label+command_name+command_args]
    /// Concatenate them so we don't have to use a HashMap<String, Vec<String>>
    pub active_commands: Arc<Mutex<HashSet<Arc<String>>>>,
    pub bash: BashPool,
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
        bash: BashPool::new(3).await?,
    };

    let app = Router::new()
        .route("/healthcheck", get(healthcheck))
        .route("/exec", post(exec))
        .route("/ws", get(websocket_handler))
        .route("/window/{label}/{action}", get(window_handler))
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

async fn exec(
    State(app_state): State<AppState>,
    Json(body): Json<ExecRequest>,
) -> impl IntoResponse {
    info!(
        "Requesting execution of command: {} {:?}",
        body.command, body.args
    );

    match app_state.bash.execute(&body.command, Some(body.args)).await {
        Ok(output) => (StatusCode::OK, output),
        Err(e) => {
            error!("Command execution failed: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Execution error: {}", e),
            )
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

struct ChildProcessHandle {
    child: tokio::process::Child,
    stream_id: Arc<String>,
}

async fn client_handler(
    fut: upgrade::UpgradeFut,
    app_state: AppState,
) -> Result<(), WebSocketError> {
    let mut ws = FragmentCollector::new(fut.await?);
    let (tx, mut rx) = mpsc::unbounded_channel::<WebsocketStreamMessage>();
    let mut label: Option<String> = None;
    let spawned_children = Arc::new(Mutex::new(Vec::<ChildProcessHandle>::new()));

    let result = loop {
        tokio::select! {
            Some(message) = rx.recv() => {
                let stringified = serde_json::to_string(&message)
                    .expect("If this fails I'll kill myself");
                let payload = Payload::Borrowed(stringified.as_bytes());

                if let Err(e) = ws.write_frame(Frame::text(payload)).await {
                    error!("Failed to send via socket: {}", e);
                    break Err(e);
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
                                debug!("Identified client with label '{}'", new_label);
                            }

                            "LISTEN" => {
                                let Some(stream_id) = parts.get(1).map(|s| s.to_string()) else {
                                    error!("Received listen signal without a stream ID");
                                    continue;
                                };
                                let stream_id = Arc::new(stream_id);

                                let Some(command) = parts.get(2) else {
                                    error!("Received listen signal without a command");
                                    continue;
                                };

                                let args = parts.iter().skip(3).copied().collect::<Vec<_>>();
                                debug!("Stream '{}'  is listening to {} {:?}", stream_id, command, args);

                                let mut lock = app_state.active_commands.lock().await;

                                if lock.contains(&*stream_id) {
                                    warn!("Stopping duplicate stream '{}' (This is normal if you are in --dev mode)", &stream_id);
                                    continue;
                                } else {
                                    lock.insert(Arc::clone(&stream_id));
                                }

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

                                let child_id = child.id();
                                debug!("Spawned child process with PID: {:?}", child_id);

                                let Some(stdout) = child.stdout.take() else {
                                    error!("Failed to capture stdout for '{} {:?}'", command, args);
                                    continue;
                                };

                                let tx_clone = tx.clone();
                                let stream_id_clone = Arc::clone(&stream_id);
                                let children_ref = Arc::clone(&spawned_children);
                                let app_state_clone = app_state.clone();

                                tokio::spawn(async move {
                                    let mut stdout_reader = BufReader::new(stdout).lines();

                                    loop {
                                        tokio::select! {
                                            Ok(Some(line)) = stdout_reader.next_line() => {
                                                let message = WebsocketStreamMessage {
                                                    stream_id: Arc::clone(&stream_id_clone),
                                                    data: line,
                                                };

                                                let display_data = if message.data.len() > 100 {
                                                    format!("{}... ({} bytes)", &message.data[..100], message.data.len())
                                                } else {
                                                    message.data.clone()
                                                };

                                                debug!("Sending {} to stream '{}'", display_data, message.stream_id);

                                                if tx_clone.send(message).is_err() {
                                                    break;
                                                }
                                            }
                                            else => break,
                                        }
                                    }

                                    let mut commands_lock = app_state_clone.active_commands.lock().await;
                                    commands_lock.remove(&*stream_id_clone);

                                    let mut children_lock = children_ref.lock().await;

                                    if let Some(pos) = children_lock.iter().position(|h| h.stream_id.as_ref() == stream_id_clone.as_ref()) {
                                        if let Some(handle) = children_lock.get_mut(pos) {

                                            debug!("Child process {:?} exited naturally", handle.child.id());
                                            let _ = handle.child.wait().await;
                                        }

                                        children_lock.remove(pos);
                                    }
                                });

                                let mut children_lock = spawned_children.lock().await;
                                children_lock.push(ChildProcessHandle {
                                    child,
                                    stream_id: Arc::clone(&stream_id),
                                });
                            }

                            "STOP_STREAM" => {
                                let Some(stream_id) = parts.get(1).map(|s| s.to_string()) else {
                                    error!("Received STOP_STREAM without stream ID");
                                    continue;
                                };

                                let mut senders_lock = app_state.websocket_senders.write().await;
                                senders_lock.remove(&stream_id);

                               let mut commands_lock = app_state.active_commands.lock().await;
                               commands_lock.remove(&stream_id);

                                // Kill the specific child process
                                let mut children_lock = spawned_children.lock().await;
                                if let Some(pos) = children_lock.iter().position(|h| h.stream_id.as_ref() == &stream_id) {
                                    if let Some(handle) = children_lock.get_mut(pos) {
                                        if let Some(pid) = handle.child.id() {

                                            debug!("Killing child process with PID: {} for stream '{}'", pid, stream_id);
                                            let _ = handle.child.kill().await;
                                        }
                                    }

                                    children_lock.remove(pos);
                                }

                                warn!("Stopped stream '{}'", stream_id);
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

    let mut children_lock = spawned_children.lock().await;

    for handle in children_lock.iter_mut() {
        if let Some(pid) = handle.child.id() {
            info!(
                "Cleaning up: killing child process with PID: {} (stream: {})",
                pid, handle.stream_id
            );

            if let Err(e) = handle.child.kill().await {
                error!("Failed to kill child process {}: {}", pid, e);
            } else {
                // Wait for the process to actually die
                let _ = handle.child.wait().await;
            }
        }
    }

    children_lock.clear();

    result
}

async fn window_handler(
    Path((label, action)): Path<(String, String)>,
    State(app_state): State<AppState>,
) {
    let request = WindowActionRequest {
        target_label: label,
        action: WindowAction::from(action),
    };

    let _ = app_state.window_tx.send(request);
}
