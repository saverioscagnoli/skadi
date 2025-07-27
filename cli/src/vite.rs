use err::SkadiError;
use tokio::fs;
use traccia::info;

static PACKAGE_JSON: &str = include_str!("../../templates/package.json");
static VITE_CONFIG_JS: &str = include_str!("../../templates/vite.config.js");

pub async fn init_workspace() -> Result<(), SkadiError> {
    let local = paths::local().ok_or(SkadiError::ViteWorkspaceInit(
        "Failed to find or create local directory".to_string(),
    ))?;

    info!("Initializing Vite workspace in {}", local.display());

    // Copy templates to the local directory
    let package_json_path = local.join("package.json");
    let vite_config_js_path = local.join("vite.config.js");

    info!("Creating package.json at {}", package_json_path.display());

    fs::write(&package_json_path, PACKAGE_JSON)
        .await
        .map_err(|e| SkadiError::ViteWorkspaceInit(e.to_string()))?;

    info!(
        "Creating vite.config.js at {}",
        vite_config_js_path.display()
    );

    fs::write(&vite_config_js_path, VITE_CONFIG_JS)
        .await
        .map_err(|e| SkadiError::ViteWorkspaceInit(e.to_string()))?;

    info!("Vite workspace initialized successfully");

    Ok(())
}
