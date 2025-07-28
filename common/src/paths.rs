use std::{fs, path::PathBuf};

use err::SkadiError;

/// Return the path to the user configuration directory
/// If the directory does not exist, it will be created.
///
/// `~/.config/skadi`
pub fn config() -> Option<PathBuf> {
    let mut user_config = dirs::config_dir()?;
    user_config.push("skadi");

    fs::create_dir_all(&user_config).ok()?;

    Some(user_config)
}

/// Return the path to the user data directory
/// If the directory does not exist, it will be created.
///
/// `~/.local/share/skadi`
pub fn local() -> Option<PathBuf> {
    let mut user_local = dirs::data_local_dir()?;
    user_local.push("skadi");

    fs::create_dir_all(&user_local).ok()?;

    Some(user_local)
}

/// Returns the path to the HTML indices directory
/// This path will contain the HTML files that vite needs to compile
/// for all the windows specified in the configuration.
/// If the directory does not exist, it will be created.
///
/// `~/.local/share/skadi/html`
pub fn html_indices() -> Option<PathBuf> {
    let mut local = local()?;
    local.push("html");

    fs::create_dir_all(&local).ok()?;

    Some(local)
}

/// Returns the path to the HTML indices directory
/// This path will contain the JSX files that vite needs to compile
/// for all the windows specified in the configuration.
/// If the directory does not exist, it will be created.
///
/// `~/.local/share/skadi/jsx`
pub fn jsx_indices() -> Option<PathBuf> {
    let mut local = local()?;
    local.push("jsx");

    fs::create_dir_all(&local).ok()?;

    Some(local)
}

/// Returns the path to the plugins directory
/// This path will contain the plugins that vite needs to compile
/// for all the windows specified in the configuration.
/// If the directory does not exist, it will be created.
///
/// `~/.config/skadi/plugins`
pub fn plugins() -> Option<PathBuf> {
    let mut config = config()?;
    config.push("plugins");

    fs::create_dir_all(&config).ok()?;

    Some(config)
}

pub fn possible_configs() -> Result<Vec<PathBuf>, SkadiError> {
    let mut paths = Vec::new();

    let d = config().ok_or(SkadiError::PathNotFound)?;

    paths.push(d.join("config.json"));
    paths.push(d.join("config.jsonc"));
    paths.push(d.join("config.json5"));

    Ok(paths)
}
