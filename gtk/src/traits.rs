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
    prelude::{GtkWindowExt, WidgetExt},
};
use gtk4_layer_shell::LayerShell;
use traccia::error;

#[cfg(debug_assertions)]
use traccia::debug;
use webkit6::prelude::WebViewExt;

pub trait GtkSetup {
    fn create_app(&self) -> gtk4::Application;
    fn setup_windows(&self, app: &gtk4::Application) -> Result<Vec<gtk4::ApplicationWindow>>;
}

impl GtkSetup for Config {
    fn create_app(&self) -> gtk4::Application {
        gtk4::Application::builder()
            .application_id(&self.app_id)
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

        for wc in &self.windows {
            let window = gtk4::ApplicationWindow::builder()
                .application(app)
                .title(&wc.label)
                .build();

            let webview = webkit6::WebView::builder()
                .web_context(&web_context)
                .build();

            let context = webview.web_context().unwrap();

            // Clone data needed inside the closure
            let dist_cloned = dist.clone();
            let label_cloned = wc.label.clone();

            context.register_uri_scheme("app", move |request| {
                let uri = request.uri();
                let binding = uri.unwrap();
                let path = binding
                    .strip_prefix("app://localhost")
                    .unwrap_or("")
                    .trim_start_matches('/');

                let file_path = if path.is_empty() {
                    // Serve the main HTML file
                    dist_cloned.join(format!("html/{}.html", label_cloned))
                } else {
                    // Serve other assets (CSS, JS, etc.)
                    dist_cloned.join(path)
                };

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
            });

            #[cfg(debug_assertions)]
            {
                if let Some(settings) = webkit6::prelude::WebViewExt::settings(&webview) {
                    settings.set_enable_developer_extras(true);
                    //   settings.set_disable_web_security(true);
                }
            }

            webview.load_uri("app://localhost");
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

            let user_content_manager = webview
                .user_content_manager()
                .expect("WebView should have a UserContentManager");

            user_content_manager.register_script_message_handler("exec", None);

            windows.push(window);
        }

        Ok(windows)
    }
}
