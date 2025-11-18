use crate::window::{WindowAction, WindowActionRequest};
use axum::{
    Router,
    extract::{ConnectInfo, Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use fastwebsockets::{
    FragmentCollector, Frame, OpCode, Payload, WebSocket, WebSocketError, upgrade,
};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::sync::Arc;
use std::{collections::HashMap, error::Error};
use std::{collections::HashSet, path::PathBuf};
use std::{env::consts::ARCH, net::SocketAddr};
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

#[derive(Clone)]
pub struct AppState {
    pub window_tx: UnboundedSender<WindowActionRequest>,
    pub websocket_senders: Arc<RwLock<HashMap<String, UnboundedSender<String>>>>,
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

#[derive(Deserialize)]
struct ExecRequest {
    command: String,
    args: Vec<String>,
}

#[derive(Serialize)]
struct ExecMessage {
    success: bool,
    stdout: String,
    stderr: String,
}

async fn exec(Json(payload): Json<ExecRequest>) -> impl IntoResponse {
    info!(
        "Executing command: {} {:?}",
        &payload.command, &payload.args
    );

    let output = Command::new(&payload.command)
        .args(&payload.args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await;

    match output {
        Ok(output) => {
            let success = output.status.success();
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            let response = ExecMessage {
                success,
                stdout,
                stderr,
            };

            Json(response)
        }

        Err(e) => {
            warn!(
                "Command {} {:?} failed: {}",
                &payload.command, &payload.args, e
            );

            let response = ExecMessage {
                success: false,
                stdout: String::new(),
                stderr: e.to_string(),
            };

            Json(response)
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
        if let Err(e) = client_handler(fut, app_state, addr.to_string()).await {
            error!("Error handling websocket from {}: {}", addr, e);
        } else {
            debug!("WebSocket connection closed: {}", addr);
        }
    });

    response
}

async fn read_frame<'a>(frame: Frame<'a>, ws: &WebSocket<String>) -> Result<u8, WebSocketError> {
    match frame.opcode {
        OpCode::Close => return Ok(1),
        OpCode::Text => {
            let text = String::from_utf8(frame.payload.to_vec()).unwrap();
            let parts = text.split_whitespace().collect::<Vec<_>>();

            if parts.is_empty() {
                error!("Received an empty message");
            }

            match parts[0] {
                "IDENTIFY" => {}
                "SEND" => {}
                p => {
                    error!("Received unknown protocol part: {}", p);
                    return Ok(1);
                }
            }

            if parts.is_empty() {
                warn!("Received an empty command. Did you use a listen hook with empty args?");
            }

            let command = vec[0];
            let args = vec.iter().skip(1).collect::<Vec<_>>();
            let mut child = match Command::new(&command)
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to spawn command: {}", e);
                    return Ok(1);
                }
            };

            let Some(stdout) = child.stdout.take() else {
                error!("Failed to take stdout");
                return Ok(1);
            };
            let mut reader = BufReader::new(stdout).lines();

            while let Ok(Some(line)) = reader.next_line().await {
                let payload = Payload::Borrowed(line.as_bytes());
                ws.write_frame(Frame::text(payload)).await?;
            }
        }

        _ => {}
    }

    Ok(0)
}

async fn client_handler(
    fut: upgrade::UpgradeFut,
    app_state: AppState,
    addr: String,
) -> Result<(), WebSocketError> {
    let mut ws = FragmentCollector::new(fut.await?);
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    loop {
        tokio::select! {
            Some(message) = rx.recv() => {
                let payload = Payload::Borrowed(message.as_bytes());

                if let Err(e) = ws.write_frame(Frame::text(payload)).await {
                    error!("Failed to send via socket: {}", e);
                }
            }

            frame_result = ws.read_frame() => {
                let frame = frame_result?;

                match read_frame(frame, &ws).await {
                    Ok(_) => continue,
                    Err(e) => {
                        error!("Websocket error: {}",e);
                        continue;
                    }
                }
            }
        }
    }
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
