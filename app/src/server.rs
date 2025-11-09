use axum::{Router, http::StatusCode, response::IntoResponse, routing::get};
use std::error::Error;
use std::path::PathBuf;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tower_http::services::ServeDir;
use traccia::{debug, info};

pub async fn start_server(
    port: u16,
    root_dir: PathBuf,
    ready_tx: oneshot::Sender<()>,
) -> Result<(), Box<dyn Error>> {
    let app = Router::new()
        .route("/healthcheck", get(healthcheck))
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
