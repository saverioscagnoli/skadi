use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use std::path::PathBuf;
use tokio::sync::oneshot;
use tower_http::services::ServeDir;
use traccia::{error, info};

use crate::{config::Paths, error::SkadiError};

pub async fn asset_server(
    port: u16,
    root_dir: PathBuf,
    ready_tx: oneshot::Sender<()>,
) -> Result<(), SkadiError> {
    let app = Router::new()
        .route("/healthcheck", get(healthcheck))
        .route("/plugins", get(plugins))
        .fallback_service(ServeDir::new(root_dir));

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

pub async fn healthcheck() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

pub async fn plugins() -> impl IntoResponse {
    let Some(config_dir) = Paths::config_dir().ok() else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Configuration directory not found",
        )
            .into_response();
    };

    let plugins_dir = config_dir.join("plugins");

    if !plugins_dir.exists() {
        return (StatusCode::NOT_FOUND, "Plugins directory does not exist").into_response();
    }

    // Send back paths of the plugins
    let paths: Vec<String> = std::fs::read_dir(plugins_dir)
        .map_err(|_| SkadiError::PathNotFound)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|entry| {
            let path = entry.path();

            // Skip .d.ts files
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.ends_with(".d.ts"))
                .unwrap_or(false)
            {
                return false;
            }

            // Check for allowed extensions
            path.extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| matches!(ext, "jsx" | "tsx" | "js" | "ts"))
                .unwrap_or(false)
        })
        .map(|entry| entry.path().to_string_lossy().into_owned())
        .collect();

    (StatusCode::OK, Json(paths)).into_response()
}
