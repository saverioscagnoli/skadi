use std::{fs, process::Stdio};
use tauri::{Emitter, WebviewWindow};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

use crate::paths::Paths;

#[tauri::command]
pub fn get_plugins() -> Result<Vec<String>, String> {
    let plugins_path = Paths::plugins().ok_or("Failed to get plugins directory")?;

    // No need to check if exists, as the directory is created in Paths::plugins()
    let entries = fs::read_dir(&plugins_path)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();

            if let Some(ext) = path.extension() {
                if ext == "jsx" || ext == "tsx" {
                    let name = path.file_name()?.to_string_lossy().to_string();
                    return Some(name);
                }
            }

            None
        })
        .collect::<Vec<_>>();

    Ok(entries)
}

#[tauri::command]
pub fn read_plugin(file_name: String) -> Result<String, String> {
    let plugins_path = Paths::plugins().ok_or("Failed to get plugins directory")?;
    let file_path = plugins_path.join(file_name);

    fs::read_to_string(file_path).map_err(|e| e.to_string())
}

/// Executes a script or an executable, sending the result back.
/// If `polls` is true, it will send back the result periodically
/// to window where the command was called; use `useTauriEvent` in the frontend to listen for the event.
///
/// Note: The `path` parameter must be valid and relative to the configuration path.
/// Example: ~/.config/skadi/assets/script.sh` -path` must be "assets/script.sh"
#[tauri::command]
pub async fn exec(
    window: WebviewWindow,
    path: String,
    is_executable: bool,
    polls: bool,
) -> Result<serde_json::Value, String> {
    let config_path = Paths::config().ok_or("Failed to get config directory")?;
    let path = config_path.join(path);
    let name = path
        .file_name()
        .ok_or("Failed to get file name from path")?
        .to_string_lossy()
        .to_string();

    let mut command = if is_executable {
        Command::new(&path)
    } else {
        let mut c = Command::new("bash");
        c.arg(&path);
        c
    };

    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = command.spawn().map_err(|e| e.to_string())?;

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let reader = BufReader::new(stdout);

    let mut lines = reader.lines();
    let mut output = String::new();

    while let Some(line) = lines.next_line().await.map_err(|e| e.to_string())? {
        output.push_str(&line);
        output.push('\n');

        if polls {
            // Send the output to the frontend
            if let Ok(json_line) = serde_json::from_str::<serde_json::Value>(&line) {
                window
                    .emit(name.as_str(), &json_line)
                    .map_err(|e| e.to_string())?;
            }
        } else {
            return Ok(serde_json::Value::String(output));
        }
    }

    // This code will only execute if polling script ends
    let status = child.wait().await.map_err(|e| e.to_string())?;

    if !status.success() {
        return Err(format!("Script execution failed with status: {}", status));
    }

    serde_json::from_str(&output).map_err(|e| format!("Failed to parse output as JSON: {}", e))
}
