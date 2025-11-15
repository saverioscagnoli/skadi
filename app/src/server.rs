use crate::window::{WindowAction, WindowActionRequest};
use axum::{
    Router,
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::sync::Arc;
use std::{collections::HashMap, error::Error};
use std::{collections::HashSet, path::PathBuf};
use tokio::net::TcpListener;
use tokio::sync::{RwLock, oneshot};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    sync::Mutex,
};
use tokio::{process::Command, sync::mpsc::UnboundedSender};
use tower_http::{
    cors::{self, CorsLayer},
    services::ServeDir,
};
use traccia::{debug, error, info, warn};

#[derive(Debug, Clone, Deserialize)]
pub struct ListenBody {
    pub script: String,
    pub args: Vec<String>,
    pub widget_label: String,
}

#[derive(Clone)]
pub struct AppState {
    pub window_tx: UnboundedSender<WindowActionRequest>,
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
        active_commands: Arc::new(Mutex::new(HashSet::new())),
    };

    let mut app = Router::new()
        .route("/healthcheck", get(healthcheck))
        .route("/exec", post(exec))
        .route("/listen", post(listen))
        .route("/window/{action}", post(window_action_handler))
        .layer(cors)
        .with_state(app_state);

    app = Router::new()
        .nest("/backend", app)
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

pub async fn listen(State(app_state): State<AppState>, Json(body): Json<ListenBody>) {
    let set_key = format!(
        "{}{}{}",
        body.widget_label,
        body.script,
        body.args.join(" ")
    );

    // Check if this listener is already active
    {
        let mut widget_commands = app_state.active_commands.lock().await;

        if widget_commands.contains(&set_key) {
            warn!(
                "Ignoring duplicate listener request for widget {} with script: {} {:?} (this is normal if you have multiple instances of the widget)",
                &body.widget_label, &body.script, &body.args
            );

            return;
        }

        let set_key = set_key.clone();
        widget_commands.insert(set_key);
    }

    let widget_label = body.widget_label.clone();
    let event_name = format!("{} {}", &body.script, &body.args.join(" "));
    let window_tx = app_state.window_tx.clone();

    tokio::spawn(async move {
        info!(
            "{} is listening to: {} {:?}",
            widget_label, event_name, &body.args
        );

        let mut child = match Command::new(&body.script)
            .args(&body.args)
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
            let action = WindowActionRequest {
                target: widget_label.clone(),
                action: WindowAction::DispatchEvent(event_name.clone(), line),
            };

            if window_tx.send(action).is_err() {
                error!("Failed to send event, receiver dropped");
                break;
            }
        }

        match child.wait().await {
            Ok(status) => debug!("Listener exited with: {}", status),
            Err(e) => error!("Error waiting for command: {}", e),
        }

        // Remove this listener from active set when it finishes
        let mut widget_commands = app_state.active_commands.lock().await;

        widget_commands.remove(&set_key);
        debug!(
            "Listener removed for widget: {} with script: {} {:?}",
            &body.widget_label, &body.script, &body.args
        );
    });
}

pub async fn window_action_handler(Path(payload): Path<String>) -> impl IntoResponse {
    debug!("window action: {}", payload);
}
