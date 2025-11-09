use std::{path::PathBuf, sync::LazyLock};
use traccia::fatal;

const USER: LazyLock<String> = LazyLock::new(|| match std::env::var("USER") {
    Ok(u) => u,
    Err(e) => {
        fatal!("Could not determine current user: {}", e);
        std::process::exit(1);
    }
});

/// Returns /home/<user>/.config/wwidgets, creating it if necessary.
///
/// In this directory, the user will store configuration files.
/// The program will crete a default config file if there is none.
pub fn config_dir() -> PathBuf {
    match dirs::config_dir() {
        Some(d) => d.join("wwidgets"),
        None => {
            let path = PathBuf::from(format!("/home/{}/.config/wwidgets", *USER));

            if let Err(e) = std::fs::create_dir_all(&path) {
                fatal!("Could not determine or create config directory: {}", e);
                std::process::exit(1);
            }

            path
        }
    }
}

/// Returns /home/<user>/.local/share/wwidgets, creating it if necessary.
///
/// In this directory, the program, will store builds for the backend
/// to serve on localhost, so the gtk webview can load them.
pub fn builds_dir() -> PathBuf {
    match dirs::data_local_dir() {
        Some(d) => d.join("wwidgets"),
        None => {
            let path = PathBuf::from(format!("/home/{}/.local/share/wwidgets", *USER));

            if let Err(e) = std::fs::create_dir_all(&path) {
                fatal!("Could not determine or create local data directory: {}", e);
                std::process::exit(1);
            }

            path
        }
    }
}

/// Returns /home/<user>/.cache/wwidgets, creating it if necessary.
///
/// In this directory, the cli program will store temporary files,
/// like js files, vite config, etc.
/// Effectively this is where vite will operate.
pub fn cache_dir() -> PathBuf {
    match dirs::cache_dir() {
        Some(d) => d.join("wwidgets"),
        None => {
            let path = PathBuf::from(format!("/home/{}/.cache/wwidgets", *USER));

            if let Err(e) = std::fs::create_dir_all(&path) {
                fatal!("Could not determine or create cache directory: {}", e);
                std::process::exit(1);
            }

            path
        }
    }
}
