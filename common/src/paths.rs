use std::{path::PathBuf, sync::LazyLock};
use traccia::fatal;

static USER: LazyLock<String> = LazyLock::new(|| match std::env::var("USER") {
    Ok(u) => u,
    Err(e) => {
        fatal!("Could not determine current user: {}", e);
        std::process::exit(1);
    }
});

/// Returns /home/$USER/.config/wwwidgets, creating it if necessary.
///
/// In this directory, the user will store configuration files.
/// The program will crete a default config file if there is none.
pub fn config_dir() -> PathBuf {
    match dirs::config_dir() {
        Some(d) => d.join("wwwidgets"),
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

/// Returns /home/$USER/.local/share/wwidgets, creating it if necessary.
///
/// In this directory, the program, will store builds for the backend
/// to serve on localhost, so the gtk webview can load them.
pub fn local_dir() -> PathBuf {
    let path = match dirs::data_local_dir() {
        Some(d) => d.join("wwwidgets"),
        None => {
            let path = PathBuf::from(format!("/home/{}/.local/share/wwidgets", *USER));

            if let Err(e) = std::fs::create_dir_all(&path) {
                fatal!("Could not determine or create local data directory: {}", e);
                std::process::exit(1);
            }

            path
        }
    };

    if !path.exists()
        && let Err(e) = std::fs::create_dir_all(&path)
    {
        fatal!("Could not determine or create local data directory: {}", e);
        std::process::exit(1);
    }

    path
}

/// Returns /tmp/wwwidgets, creating it if necessary.
///
/// This directory is used to store temporary files,
/// and the webview will have access to it using tmp://file-name,
/// so that the user can read files from it
///
/// The notification daemon in the same project uses it to store
/// notifciation images.
pub fn tmp_dir() -> PathBuf {
    let path = PathBuf::from("/tmp/wwwidgets");

    if !path.exists()
        && let Err(e) = std::fs::create_dir_all(&path)
    {
        fatal!("Could not determine or create local data directory: {}", e);
        std::process::exit(1);
    }

    path
}
