use common::{
    config::Config,
    debug_mode, dev_mode,
    io::{Io, OutputMode, SpawnOptions},
};
use gtk4::{
    gdk::{
        RGBA,
        prelude::{DisplayExt, MonitorExt},
    },
    gio::{
        ApplicationFlags,
        prelude::{ApplicationExt, ApplicationExtManual, ListModelExtManual},
    },
    glib::{self, LogWriterOutput},
    prelude::{GtkWindowExt, WidgetExt},
};
use gtk4_layer_shell::LayerShell;
use serde::Deserialize;
use serde_json::json;
use std::{collections::HashMap, path::Path};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Child,
    sync::mpsc::{self, error::TryRecvError},
};
use traccia::{debug, error, fatal, info, warn};
use webkit6::{WebContext, WebView, prelude::WebViewExt};

#[derive(Debug, Deserialize)]
struct ExecMessage {
    id: String,
    action: String,
    script: String,
    polls: Option<bool>,
    args: Option<Vec<String>>,
}

fn disable_gtk_logs() {
    debug!("Disabling GTK logs");
    gtk4::glib::log_set_writer_func(|_log_domain, _log_level| LogWriterOutput::Unhandled);
}

pub fn run<P: AsRef<Path>>(config: &Config, root: &P) {
    let root = root.as_ref();

    disable_gtk_logs();

    if let Err(e) = gtk4::init() {
        fatal!("Failed to initialize GTK: {}", e);
        return;
    }

    let app = gtk4::Application::new(Some("com.skadi.dev"), ApplicationFlags::FLAGS_NONE);

    app.connect_startup(move |_| {
        let provider = gtk4::CssProvider::new();
        let css_str = r"
            window {
                background-color: transparent;
            }
        ";

        debug!("Injecting transparent background CSS");
        provider.load_from_string(css_str);

        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        if let Some(settings) = gtk4::Settings::default() {
            settings.set_gtk_theme_name(Some(""));
            settings.set_gtk_icon_theme_name(Some(""));
        }
    });

    let root = root.to_path_buf();
    let config = config.clone();

    app.connect_activate(move |app| {
        let windows = match setup_windows(&app, &config, &root) {
            Ok(w) => w,
            Err(e) => {
                fatal!("Failed to setup windows: {}", e);
                return;
            }
        };

        for w in windows {
            info!(
                "Created window '{}' {}x{}",
                w.title().unwrap_or("Untitled".into()),
                w.width_request(),
                w.height_request()
            );

            w.present();
        }
    });

    app.run_with_args(&Vec::<String>::new());
}

fn setup_windows(
    app: &gtk4::Application,
    config: &Config,
    root: &Path,
) -> Result<Vec<gtk4::ApplicationWindow>, Box<dyn std::error::Error>> {
    let mut windows = Vec::new();
    let Some(display) = gtk4::gdk::Display::default() else {
        return Err("Could not connect to a display".into());
    };

    debug!("Using display: {}", display.name());

    let mut monitor_map = HashMap::new();

    for (i, m) in display
        .monitors()
        .iter::<gtk4::gdk::Monitor>()
        .filter_map(Result::ok)
        .enumerate()
    {
        let geometry = m.geometry();

        debug!(
            "Monitor {}: {} - {}x{} @ ({}, {})",
            i + 1,
            m.connector().unwrap_or("Unknown".into()),
            geometry.width(),
            geometry.height(),
            geometry.x(),
            geometry.y()
        );

        let Some(connector) = m.connector() else {
            warn!("Monitor {} has no connector", i);
            continue;
        };

        monitor_map.insert(connector.to_string(), m);
    }

    let context = WebContext::new();
    let dist = root.join("dist");

    debug!("Using dist directory: {}", dist.display());

    // Custom uri scheme for serving files
    context.register_uri_scheme("skadi", move |req| {
        let Some(uri) = req.uri() else {
            error!("Failed to get URI from request");
            return;
        };

        let path = uri
            .strip_prefix("skadi://localhost/")
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

        debug!(
            "Serving file: {:?}",
            file_path.file_name().unwrap_or_default()
        );

        match std::fs::read(&file_path) {
            Ok(data) => {
                let mime_type = mime_guess::from_path(&file_path)
                    .first_or_octet_stream()
                    .to_string();

                let stream =
                    gtk4::gio::MemoryInputStream::from_bytes(&gtk4::glib::Bytes::from(&data));

                req.finish(&stream, data.len() as i64, Some(&mime_type));
            }

            Err(e) => {
                eprintln!("Failed to read file {:?}: {}", file_path, e);

                let mut error = gtk4::glib::Error::new(
                    gtk4::gio::IOErrorEnum::NotFound,
                    &format!("File not found: {:?}", file_path),
                );

                req.finish_error(&mut error);
            }
        }
    });

    for w in &config.windows {
        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .title(&w.label)
            .build();

        // Use the same context for all webviews to save resources
        let webview = WebView::builder().web_context(&context).build();

        // Disable the default context menu if not debugging
        if debug_mode() {
            debug!("Context menu is enabled for debugging");
        } else {
            webview.connect_context_menu(|_, _, _| true);
        }

        if debug_mode() {
            if let Some(settings) = webkit6::prelude::WebViewExt::settings(&webview) {
                settings.set_enable_developer_extras(true);
            }
        }

        if dev_mode() {
            // Load html from vite url server
            webview.load_uri(&format!("http://localhost:5173/html/{}.html", w.label));
        } else {
            // Load html from custom uri scheme
            webview.load_uri(&format!("skadi://localhost/{}", w.label));
        }

        webview.set_background_color(&RGBA::TRANSPARENT);

        window.set_child(Some(&webview));

        // Check if we need layer shell functionality or regular window positioning
        let use_layer_shell = w.layer != common::config::Layer::Top
            || w.exclusive
            || w.margin_top.is_some()
            || w.margin_bottom.is_some()
            || w.margin_left.is_some()
            || w.margin_right.is_some()
            || (w.x != 0 || w.y != 0); // Use layer shell for positioning too

        // Find the specified monitor
        // (e.g. "eDP-1", "HDMI-A-1", etc.)
        let Some(monitor) = monitor_map.get(&w.monitor) else {
            error!(
                "Monitor '{}' not found when trying to create window '{}'",
                w.monitor, w.label
            );

            continue;
        };

        let geometry = monitor.geometry();

        let width = w.width.as_pixel(geometry.width());
        let height = w.height.as_pixel(geometry.height());

        window.set_width_request(width);
        window.set_height_request(height);

        if use_layer_shell {
            // Use layer shell for overlay functionality
            window.init_layer_shell();
            window.set_layer(w.layer.into());

            if w.exclusive {
                window.auto_exclusive_zone_enable();
            }

            // Use positioning with x/y coordinates if provided
            if w.x != 0 || w.y != 0 {
                w.anchor.apply_with_position(&window, w.x, w.y);
            } else {
                w.anchor.apply(&window);
            }
        }

        // Set after initializing layer shell
        // Otherwise it won't work correctly
        window.set_monitor(Some(monitor));

        if let Some(margin) = w.margin_top {
            window.set_margin(gtk4_layer_shell::Edge::Top, margin);
        }

        if let Some(margin) = w.margin_bottom {
            window.set_margin(gtk4_layer_shell::Edge::Bottom, margin);
        }

        if let Some(margin) = w.margin_left {
            window.set_margin(gtk4_layer_shell::Edge::Left, margin);
        }

        if let Some(margin) = w.margin_right {
            window.set_margin(gtk4_layer_shell::Edge::Right, margin);
        }

        if w.layer == common::config::Layer::Background {
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
                    let message: ExecMessage = match serde_json::from_str(&message.to_string()) {
                        Ok(m) => m,
                        Err(e) => {
                            error!("Failed to parse message: {}", e);
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

async fn handle_frontend_message(message: &ExecMessage, webview: &webkit6::WebView) {
    if message.action != "exec" {
        eprintln!(
            "How did you even get here? Action '{}' is not supported",
            message.action
        );
        return;
    }

    // Vec<String> -> Vec<&str>
    let args = message
        .args
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>();

    let child = match Io::spawn_child(
        &message.script,
        &args,
        SpawnOptions {
            stdout: OutputMode::Pipe,
            stderr: OutputMode::Pipe,
            ..Default::default()
        },
    )
    .await
    {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to spawn child process: {}", e);
            return;
        }
    };

    if message.polls.unwrap_or(false) {
        handle_polling_command(message, child, webview).await;
    } else {
        handle_oneshot_command(message, child, webview).await;
    }
}

async fn handle_polling_command(message: &ExecMessage, mut child: Child, webview: &WebView) {
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

                if debug_mode() {
                    debug!(
                        "dispatching '{}' with payload: {}",
                        script,
                        serde_json::to_string_pretty(&line_content)
                            .unwrap_or_else(|_| "null".to_string())
                    );
                }

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

async fn handle_oneshot_command(message: &ExecMessage, child: Child, webview: &webkit6::WebView) {
    let output = match child.wait_with_output().await {
        Ok(o) => o,
        Err(e) => {
            eprintln!("Failed to read output: {}", e);
            return;
        }
    };

    let data = String::from_utf8_lossy(&output.stdout).to_string();

    let response = json!({
        "success": output.status.success(),
        "data": data,
    });

    debug!("exec response: {}", response);

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
