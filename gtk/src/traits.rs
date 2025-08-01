use anyhow::{Result, anyhow};
use common::{
    config::{Config, Layer},
    paths,
};
use gtk4::{
    gdk::{
        Display, RGBA,
        prelude::{DisplayExt, MonitorExt},
    },
    gio::prelude::ListModelExtManual,
    glib::{self},
    prelude::{GtkWindowExt, WidgetExt},
};
use gtk4_layer_shell::LayerShell;
use serde::Deserialize;
use serde_json::json;
use std::{
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    sync::mpsc,
    thread,
};
use traccia::error;
use webkit6::prelude::WebViewExt;

#[cfg(debug_assertions)]
use traccia::debug;

#[derive(Debug, Deserialize)]
struct Message {
    id: String,
    action: String,
    script: String,
    polls: Option<bool>,
    is_executable: Option<bool>,
    args: Option<Vec<String>>,
}

pub trait GtkSetup {
    fn create_app(&self) -> gtk4::Application;
    fn setup_windows(&self, app: &gtk4::Application) -> Result<Vec<gtk4::ApplicationWindow>>;
}

impl GtkSetup for Config {
    fn create_app(&self) -> gtk4::Application {
        gtk4::Application::builder()
            .application_id("com.skadi.app")
            .build()
    }

    fn setup_windows(&self, app: &gtk4::Application) -> Result<Vec<gtk4::ApplicationWindow>> {
        let mut windows = Vec::new();

        let Some(display) = Display::default() else {
            return Err(anyhow!("Could not connect to a display"));
        };

        let monitors = display.monitors();

        #[cfg(debug_assertions)]
        debug!("Developer tools are enabled");

        let dist = paths::dist()
            .ok_or_else(|| anyhow!("Could not find or create distribution directory"))?;

        let web_context = webkit6::WebContext::new();

        // Register the URI scheme handler ONCE, outside the window loop
        web_context.register_uri_scheme("app", {
            let dist_clone = dist.clone();
            move |request| {
                let uri = request.uri();
                let binding = uri.unwrap();
                let full_path = binding
                    .strip_prefix("app://localhost/")
                    .unwrap_or("")
                    .trim_start_matches('/');

                let file_path = if full_path.contains('/') {
                    // This is an asset request like "topbar/styles.css"
                    let parts: Vec<&str> = full_path.splitn(2, '/').collect();
                    let asset_path = parts[1];
                    // Assets are in the dist/assets/ directory
                    dist_clone.join(format!("assets/{}", asset_path))
                } else if full_path.ends_with(".css")
                    || full_path.ends_with(".js")
                    || full_path.ends_with(".png")
                    || full_path.ends_with(".jpg")
                {
                    // Direct asset request like "styles.css"
                    dist_clone.join(format!("assets/{}", full_path))
                } else if !full_path.is_empty() {
                    // HTML request like "topbar" or
                    // HTML files are in dist/html/ and have .html extension
                    dist_clone.join(format!("html/{}.html", full_path))
                } else {
                    // Fallback
                    dist_clone.join("html/index.html")
                };

                #[cfg(debug_assertions)]
                debug!("Serving file: {:?}", file_path);

                match std::fs::read(&file_path) {
                    Ok(data) => {
                        let mime_type = mime_guess::from_path(&file_path)
                            .first_or_octet_stream()
                            .to_string();
                        let stream = gtk4::gio::MemoryInputStream::from_bytes(
                            &gtk4::glib::Bytes::from(&data),
                        );
                        request.finish(&stream, data.len() as i64, Some(&mime_type));
                    }
                    Err(e) => {
                        eprintln!("Failed to read file {:?}: {}", file_path, e);
                        let mut error = gtk4::glib::Error::new(
                            gtk4::gio::IOErrorEnum::NotFound,
                            &format!("File not found: {:?}", file_path),
                        );
                        request.finish_error(&mut error);
                    }
                }
            }
        });

        for wc in &self.windows {
            let window = gtk4::ApplicationWindow::builder()
                .application(app)
                .title(&wc.label)
                .build();

            let webview = webkit6::WebView::builder()
                .web_context(&web_context)
                .build();

            #[cfg(debug_assertions)]
            {
                if let Some(settings) = webkit6::prelude::WebViewExt::settings(&webview) {
                    settings.set_enable_developer_extras(true);
                }
            }

            webview.load_uri(&format!("app://localhost/{}", wc.label));
            webview.set_background_color(&RGBA::TRANSPARENT);

            window.set_child(Some(&webview));

            window.init_layer_shell();
            window.set_layer(wc.layer.into());
            // Find the specified monitor
            // (e.g. "eDP-1", "HDMI-A-1", etc.)
            let monitor = monitors
                .iter()
                .filter_map(Result::ok)
                .find(|m: &gtk4::gdk::Monitor| {
                    if let Some(connector) = m.connector() {
                        connector == wc.monitor
                    } else {
                        false
                    }
                });

            let Some(monitor) = monitor else {
                error!(
                    "Monitor '{}' not found when trying to create window '{}'",
                    wc.monitor, wc.label
                );
                continue;
            };

            window.set_monitor(Some(&monitor));

            let geometry = monitor.geometry();

            let width = wc.width.as_pixel(geometry.width());
            let height = wc.height.as_pixel(geometry.height());

            window.set_width_request(width);
            window.set_height_request(height);

            if wc.exclusive {
                window.auto_exclusive_zone_enable();
            }

            wc.anchor.apply(&window);

            if let Some(margin) = wc.margin_top {
                window.set_margin(gtk4_layer_shell::Edge::Top, margin);
            }
            if let Some(margin) = wc.margin_bottom {
                window.set_margin(gtk4_layer_shell::Edge::Bottom, margin);
            }
            if let Some(margin) = wc.margin_left {
                window.set_margin(gtk4_layer_shell::Edge::Left, margin);
            }
            if let Some(margin) = wc.margin_right {
                window.set_margin(gtk4_layer_shell::Edge::Right, margin);
            }

            if matches!(wc.layer, Layer::Background) {
                // Set keyboard mode to none so it doesn't interfere with other windows
                window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);

                window.set_exclusive_zone(-1); // Changed from 0 to -1
                window.set_anchor(gtk4_layer_shell::Edge::Top, true);
                window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
                window.set_anchor(gtk4_layer_shell::Edge::Left, true);
                window.set_anchor(gtk4_layer_shell::Edge::Right, true);
            }

            window.set_resizable(false);
            window.set_decorated(false);

            let ucm = webview
                .user_content_manager()
                .expect("WebView should have a UserContentManager");

            ucm.register_script_message_handler("exec", None);

            ucm.connect_script_message_received(
                Some("exec"),
                glib_macros::clone!(
                    #[weak]
                    webview,
                    move |_, message| {
                        let message: Message = match serde_json::from_str(&message.to_string()) {
                            Ok(m) => m,
                            Err(e) => {
                                eprintln!("Failed to parse message: {}", e);
                                return;
                            }
                        };

                        handle_frontend_message(&message, &webview);
                    }
                ),
            );

            windows.push(window);
        }

        Ok(windows)
    }
}

fn spawn_process(message: &Message) -> Result<Child, std::io::Error> {
    let script = message.script.clone();
    if message.is_executable.unwrap_or(false) {
        Command::new(script)
            .args(message.args.clone().unwrap_or_default())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(script)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }
}

fn handle_frontend_message(message: &Message, webview: &webkit6::WebView) {
    if message.action != "exec" {
        eprintln!(
            "How did you even get here? Action '{}' is not supported",
            message.action
        );
        return;
    }

    let mut child = match spawn_process(message) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to execute script: {}", e);
            return;
        }
    };

    if message.polls.unwrap_or(false) {
        // Handle polling command - spawn thread to read continuous output
        let script_name = message.script.clone();

        // Create a channel to send data back to main thread
        let (tx, rx) = mpsc::channel::<String>();

        // Take stdout from child before moving into thread
        if let Some(stdout) = child.stdout.take() {
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    match line {
                        Ok(line_content) => {
                            if tx.send(line_content).is_err() {
                                // Receiver has been dropped, stop sending
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("Error reading from polling command: {}", e);
                            break;
                        }
                    }
                }
            });
        }

        // Clone webview for the polling loop
        let webview_clone = webview.clone();

        // Set up a recurring idle callback to check for messages
        glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
            match rx.try_recv() {
                Ok(line_content) => {
                    let js_code = format!(
                        "window.dispatchEvent(new CustomEvent('{}', {{ detail: {} }}));",
                        script_name,
                        serde_json::to_string(&line_content).unwrap_or_else(|_| "null".to_string())
                    );
                    webview_clone.evaluate_javascript(
                        &js_code,
                        None,
                        None,
                        None::<&gtk4::gio::Cancellable>,
                        |_| {},
                    );
                    glib::ControlFlow::Continue
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // No message available, continue checking
                    glib::ControlFlow::Continue
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    // Sender has been dropped, stop the timeout
                    glib::ControlFlow::Break
                }
            }
        });
    } else {
        // Handle one-shot command as before
        let output = match child.wait_with_output() {
            Ok(o) => o,
            Err(e) => {
                eprintln!("Failed to read output: {}", e);
                return;
            }
        };

        let response = match serde_json::to_string(&String::from_utf8_lossy(&output.stdout)) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Failed to serialize output: {}", e);
                return;
            }
        };

        let response = json!({
            "success": output.status.success(),
            "data": response,
        });

        // Send response back to frontend
        let js_code = format!("window.callbackHandler('{}', {});", message.id, response);
        webview.evaluate_javascript(
            &js_code,
            None,
            None,
            None::<&gtk4::gio::Cancellable>,
            |_| {},
        );
    }
}
