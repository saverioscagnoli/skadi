use std::{any::Any, fs, path::PathBuf, process::Command, sync::LazyLock, thread};

use gtk::{
    gdk::{traits::MonitorExt, Display, WindowTypeHint},
    traits::{ContainerExt, GtkWindowExt, WidgetExt},
};
use gtk_layer_shell::{Edge, Layer, LayerShell};
use tauri::{AppHandle, Emitter, Manager};
use traccia::{fatal, info};

const CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut path = dirs::config_dir().expect("Failed to get config directory");
    path.push("skadi");
    path
});

const PLUGIN_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut path = CONFIG_PATH.clone();
    path.push("plugins");
    path
});

const SCRIPTS_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let mut path = CONFIG_PATH.clone();
    path.push("scripts");
    path
});

#[tauri::command]
fn get_plugin_files() -> Result<Vec<String>, String> {
    let mut config_path = dirs::config_dir().ok_or("Failed to get config directory")?;
    config_path.push("skadi/plugins");

    if !config_path.exists() {
        fs::create_dir_all(&config_path).map_err(|e| e.to_string())?;
        return Ok(vec![]);
    }

    let entries = fs::read_dir(config_path)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "jsx" || ext == "tsx" {
                    return Some(path.file_name()?.to_string_lossy().to_string());
                }
            }
            None
        })
        .collect();

    Ok(entries)
}

#[tauri::command]
fn read_plugin_file(filename: String) -> Result<String, String> {
    let mut config_path = dirs::config_dir().ok_or("Failed to get config directory")?;
    config_path.push("skadi/plugins");
    config_path.push(&filename);

    fs::read_to_string(config_path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn exec(
    app: AppHandle,
    script: String,
    is_executable: bool,
    polls: bool,
) -> Result<serde_json::Value, String> {
    let script_path = SCRIPTS_PATH.join(script.clone());
    if !script_path.exists() {
        return Err(format!(
            "Script file does not exist: {}",
            script_path.display()
        ));
    }

    if polls {
        // Stream output line by line
        exec_with_polling(&app, script_path, is_executable).await
    } else {
        // Original behavior - wait for completion
        exec_without_polling(script_path, is_executable).await
    }
}

async fn exec_with_polling(
    app: &AppHandle,
    script_path: PathBuf,
    is_executable: bool,
) -> Result<serde_json::Value, String> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;
    let sc = script_path.clone();
    let name = sc
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("unknown script");

    let mut cmd = if is_executable {
        Command::new(script_path)
    } else {
        let mut c = Command::new("bash");
        c.arg(script_path);
        c
    };

    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let mut child = cmd.spawn().map_err(|e| e.to_string())?;

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();

    let mut all_output = String::new();

    // Read lines as they come
    while let Some(line) = lines.next_line().await.map_err(|e| e.to_string())? {
        all_output.push_str(&line);
        all_output.push('\n');

        // Try to parse as JSON and emit if valid
        if let Ok(json_line) = serde_json::from_str::<serde_json::Value>(&line) {
            app.emit(name, &json_line).map_err(|e| e.to_string())?;
        }
    }

    // Wait for the process to complete
    let status = child.wait().await.map_err(|e| e.to_string())?;

    if !status.success() {
        return Err(format!("Script exited with code: {:?}", status.code()));
    }

    // Try to parse the complete output as JSON
    serde_json::from_str(&all_output)
        .map_err(|e| format!("Failed to parse complete JSON output: {}", e))
}

async fn exec_without_polling(
    script_path: PathBuf,
    is_executable: bool,
) -> Result<serde_json::Value, String> {
    let output = if is_executable {
        tokio::process::Command::new(script_path)
            .output()
            .await
            .map_err(|e| e.to_string())?
    } else {
        tokio::process::Command::new("bash")
            .arg(script_path)
            .output()
            .await
            .map_err(|e| e.to_string())?
    };

    let stdout = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
    serde_json::from_str(&stdout).map_err(|e| format!("Failed to parse JSON output: {}", e))
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            exec,
            get_plugin_files,
            read_plugin_file
        ])
        .setup(|app| {
            let Some(tauri_window) = app.get_webview_window("main") else {
                fatal!("Failed to get the main webview window");
                std::process::exit(1);
            };

            if let Err(e) = tauri_window.hide() {
                fatal!("Failed to destroy the Tauri webview window: {}", e);
                std::process::exit(1);
            }

            let Ok(tauri_gtk_window) = tauri_window.gtk_window() else {
                fatal!("Failed to get the GTK window from the Tauri webview window");
                std::process::exit(1);
            };

            let Some(gtk_app) = tauri_gtk_window.application() else {
                fatal!("Failed to get the GTK application from the Tauri GTK window");
                std::process::exit(1);
            };

            let gtk_window = gtk::ApplicationWindow::new(&gtk_app);

            let Ok(vbox) = tauri_window.default_vbox() else {
                fatal!("Failed to get the VBox from the Tauri GTK window");
                std::process::exit(1);
            };

            tauri_gtk_window.remove(&vbox);
            gtk_window.add(&vbox);

            gtk_window.set_app_paintable(true);
            gtk_window.init_layer_shell();

            gtk_window.set_layer(Layer::Top);

            if let Some(display) = Display::default() {
                let monitor = display
                    .primary_monitor()
                    .unwrap_or_else(|| display.monitor(0).expect("No monitors found."));

                let width = monitor.geometry().width();

                gtk_window.set_width_request(width);
                // TODO: parse height from config file
                gtk_window.set_height_request(30);
                gtk_window.set_exclusive_zone(30);

                gtk_window.set_anchor(Edge::Top, true);
                gtk_window.set_anchor(Edge::Left, true);
                gtk_window.set_anchor(Edge::Right, true);

                // Set window parameters
                gtk_window.set_keep_above(true);
                gtk_window.set_resizable(false);

                // Dock the window
                gtk_window.set_type_hint(WindowTypeHint::Dock);

                // Show the gtk window
                gtk_window.show_all();
            } else {
                fatal!("No displays were found. Aborting.");
                std::process::exit(1);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
