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
    gio::prelude::{ListModelExt, ListModelExtManual},
    glib::{self, object::Cast},
    prelude::{GtkWindowExt, WidgetExt},
};
use gtk4_layer_shell::LayerShell;
use serde::Deserialize;
use serde_json::json;
use std::{collections::HashMap, process::Stdio};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    sync::mpsc::{self, error::TryRecvError},
};
use traccia::debug;
use traccia::error;
use webkit6::prelude::WebViewExt;

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
    fn setup_windows(
        &self,
        app: &gtk4::Application,
        debug: bool,
    ) -> Result<Vec<gtk4::ApplicationWindow>>;
}

impl GtkSetup for Config {
    fn create_app(&self) -> gtk4::Application {
        gtk4::Application::builder()
            .application_id("com.skadi.app")
            .build()
    }

    fn setup_windows(
        &self,
        app: &gtk4::Application,
        debug: bool,
    ) -> Result<Vec<gtk4::ApplicationWindow>> {
        let mut windows = Vec::new();

        let Some(display) = Display::default() else {
            return Err(anyhow!("Could not connect to a display"));
        };

        let monitors = display.monitors();
        let mut monitor_map = HashMap::new();

        for m in monitors.iter::<gtk4::gdk::Monitor>().filter_map(Result::ok) {
            if cfg!(debug_assertions) || debug {
                let geometry = m.geometry();

                debug!(
                    "Monitor: {} - {}x{} @ ({}, {})",
                    m.connector().unwrap_or("Unknown".into()),
                    geometry.width(),
                    geometry.height(),
                    geometry.x(),
                    geometry.y()
                );
            }

            monitor_map.insert(m.connector().unwrap_or("unknown".into()).to_string(), m);
        }

        if cfg!(debug_assertions) || debug {
            for i in 0..monitors.n_items() {
                if let Some(monitor) = monitors.item(i) {
                    let m = monitor
                        .downcast_ref::<gtk4::gdk::Monitor>()
                        .expect("Item should be a Monitor");
                    let g = m.geometry();

                    debug!(
                        "Monitor: {} - {}x{} @ ({}, {})",
                        m.connector().unwrap_or("Unknown".into()),
                        g.width(),
                        g.height(),
                        g.x(),
                        g.y()
                    );
                }
            }
        }

        let dist = paths::dist()
            .ok_or_else(|| anyhow!("Could not find or create distribution directory"))?;

        let web_context = webkit6::WebContext::new();

        // Register URI schemes for serving files without creating a server
        web_context.register_uri_scheme("app", move |request| {
            let Some(uri) = request.uri() else {
                error!("Failed to get URI from request");
                return;
            };

            let path = uri
                .strip_prefix("app://localhost/")
                .unwrap_or("")
                .trim_start_matches('/');

            let file_path;

            if path.contains('/') {
                // Asset request like "topbar/styles.css"
                let parts = path.splitn(2, '/').collect::<Vec<_>>();
                let asset_path = parts.get(1).unwrap_or(&"");

                file_path = dist.join(format!("assets/{}", asset_path));
            } else if !path.is_empty() {
                // HTML request like "topbar" or
                // HTML files are in dist/html/ and have .html extension
                file_path = dist.join(format!("html/{}.html", path));
            } else {
                // Fallback - asset file
                file_path = dist.join(format!("assets/{}", path));
            }

            if cfg!(debug_assertions) || debug {
                debug!(
                    "Serving file: {:?}",
                    file_path.file_name().unwrap_or_default()
                );
            }

            match std::fs::read(&file_path) {
                Ok(data) => {
                    let mime_type = mime_guess::from_path(&file_path)
                        .first_or_octet_stream()
                        .to_string();

                    let stream =
                        gtk4::gio::MemoryInputStream::from_bytes(&gtk4::glib::Bytes::from(&data));

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
        });

        for wc in &self.windows {
            let window = gtk4::ApplicationWindow::builder()
                .application(app)
                .title(&wc.label)
                .build();

            // Use the same context for all webviews to save resources
            let webview = webkit6::WebView::builder()
                .web_context(&web_context)
                .build();

            // Disable the default context menu if not debugging
            if !cfg!(debug_assertions) && !debug {
                webview.connect_context_menu(|_, _, _| true);
            } else {
                debug!("Context menu is enabled for debugging");
            }

            if cfg!(debug_assertions) || debug {
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
            let Some(monitor) = monitor_map.get(&wc.monitor) else {
                error!(
                    "Monitor '{}' not found when trying to create window '{}'",
                    wc.monitor, wc.label
                );

                continue;
            };

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

            if wc.layer == Layer::Background {
                // Set keyboard mode to none so it doesn't interfere with other windows
                window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);

                window.set_exclusive_zone(-1);
                window.set_anchor(gtk4_layer_shell::Edge::Top, true);
                window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
                window.set_anchor(gtk4_layer_shell::Edge::Left, true);
                window.set_anchor(gtk4_layer_shell::Edge::Right, true);
            }

            window.set_resizable(false);
            window.set_decorated(false);

            let Some(ucm) = webview.user_content_manager() else {
                error!("Failed to get UserContentManager for WebView");
                continue;
            };

            ucm.register_script_message_handler("exec", None);

            // Handle exec function from frontend
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

                        glib::spawn_future_local(async move {
                            handle_frontend_message(&message, &webview).await;
                        });
                    }
                ),
            );

            windows.push(window);
        }

        Ok(windows)
    }
}

async fn spawn_process_async(message: &Message) -> Result<Child> {
    if message.is_executable.unwrap_or(false) {
        Ok(Command::new(&message.script)
            .args(message.args.clone().unwrap_or_default())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?)
    } else {
        Ok(Command::new("sh")
            .arg("-c")
            .arg(&message.script)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?)
    }
}

async fn handle_frontend_message(message: &Message, webview: &webkit6::WebView) {
    if message.action != "exec" {
        eprintln!(
            "How did you even get here? Action '{}' is not supported",
            message.action
        );
        return;
    }

    let child = match spawn_process_async(message).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to execute script: {}", e);
            return;
        }
    };

    if message.polls.unwrap_or(false) {
        handle_polling_command(message, child, webview).await;
    } else {
        handle_oneshot_command(message, child, webview).await;
    }
}

async fn handle_polling_command(message: &Message, mut child: Child, webview: &webkit6::WebView) {
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    // Take stdout from child
    if let Some(stdout) = child.stdout.take() {
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                if tx.send(line).is_err() {
                    // Receiver has been dropped, stop sending
                    break;
                }
            }
        });
    }

    // Clone webview for the polling loop (this stays on the main thread)
    let webview = webview.clone();
    let script = message.script.clone();

    // Use glib's timeout to poll the Tokio channel from the main thread
    glib::timeout_add_local(std::time::Duration::from_millis(50), move || {
        match rx.try_recv() {
            Ok(line_content) => {
                let js_code = format!(
                    "window.dispatchEvent(new CustomEvent('{}', {{ detail: {} }}));",
                    script,
                    serde_json::to_string(&line_content).unwrap_or_else(|_| "null".to_string())
                );
                webview.evaluate_javascript(
                    &js_code,
                    None,
                    None,
                    None::<&gtk4::gio::Cancellable>,
                    |_| {},
                );
                glib::ControlFlow::Continue
            }
            Err(TryRecvError::Empty) => {
                // No message available, continue checking
                glib::ControlFlow::Continue
            }
            Err(TryRecvError::Disconnected) => {
                // Sender has been dropped, stop the timeout
                glib::ControlFlow::Break
            }
        }
    });
}

async fn handle_oneshot_command(message: &Message, child: Child, webview: &webkit6::WebView) {
    let output = match child.wait_with_output().await {
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
