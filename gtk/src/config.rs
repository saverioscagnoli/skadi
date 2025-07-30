use crate::events::{EventEmitter, SendWebView};
use err::SkadiError;
use gtk4::{
    gdk::{
        prelude::{DisplayExt, MonitorExt},
        Display, RGBA,
    },
    gio::prelude::ListModelExtManual,
    glib::{object::ObjectExt, value::ToValue},
    prelude::{GtkWindowExt, WidgetExt},
};
use gtk4_layer_shell::LayerShell;
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};
use std::{collections::HashMap, fmt, fs, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;
use traccia::{debug, error, info};
use webkit6::prelude::WebViewExt;

pub struct Paths;

impl Paths {
    pub fn config_dir() -> Result<PathBuf, SkadiError> {
        match dirs::config_dir() {
            Some(mut path) => {
                path.push("skadi");

                if !path.exists() {
                    fs::create_dir_all(&path)?;
                }

                Ok(path)
            }

            None => Err(SkadiError::PathNotFound),
        }
    }

    pub fn possible_configs() -> Result<Vec<PathBuf>, SkadiError> {
        let mut paths = Vec::new();

        let d = Self::config_dir()?;

        paths.push(d.join("config.json"));
        paths.push(d.join("config.jsonc"));
        paths.push(d.join("config.json5"));

        Ok(paths)
    }
}




    pub fn create_app(&self) -> gtk4::Application {
        gtk4::Application::builder()
            .application_id(&self.app_id)
            .build()
    }

    pub async fn setup_windows(
        &self,
        app: &gtk4::Application,
        event_emitter: EventEmitter,
    ) -> Result<Vec<gtk4::ApplicationWindow>, SkadiError> {
        let mut windows = Vec::new();

        let Some(display) = Display::default() else {
            return Err(SkadiError::WindowCreation(
                "Could not connect to a display".to_string(),
            ));
        };

        let monitors = display.monitors();

        #[cfg(debug_assertions)]
        {
            debug!("Developer tools are enabled")
        }

        for config in &self.windows {
            let window = gtk4::ApplicationWindow::builder()
                .application(app)
                .title(&config.label)
                .build();

            let webview = webkit6::WebView::new();

            // Enable web inspector for debugging
            // Will not be enabled in release builds
            #[cfg(debug_assertions)]
            {
                if let Some(settings) = webkit6::prelude::WebViewExt::settings(&webview) {
                    settings.set_enable_developer_extras(true);
                }
            }

            // At this point, the server will be already started
            // So, load the local server uri to the webview
            let uri = format!("http://localhost:{}/html/{}.html", self.port, config.label);

            webview.load_uri(&uri);

            // Set the background color to be transparent
            // So the gtk window will be transparent, and things like
            // border radius can be applied directly from the frontend CSS
            webview.set_background_color(&RGBA::TRANSPARENT);

            // Display the webview in the window
            window.set_child(Some(&webview));

            // Initialize layer shell protocol for the window
            // This allows to dock the window, set it as a panel, etc.
            window.init_layer_shell();
            window.set_layer(config.layer.into());

            // Find the specified monitor
            // (e.g. "eDP-1", "HDMI-A-1", etc.)
            let monitor = monitors
                .iter()
                .filter_map(Result::ok)
                .find(|m: &gtk4::gdk::Monitor| {
                    if let Some(connector) = m.connector() {
                        connector == config.monitor
                    } else {
                        false
                    }
                });

            let Some(monitor) = monitor else {
                error!(
                    "Monitor '{}' not found when trying to create window '{}'",
                    config.monitor, config.label
                );
                continue;
            };

            window.set_monitor(Some(&monitor));

            let geometry = monitor.geometry();

            let width = config.width.as_pixel(geometry.width());
            let height = config.height.as_pixel(geometry.height());

            window.set_width_request(width);
            window.set_height_request(height);

            if config.exclusive {
                window.auto_exclusive_zone_enable();
            }

            config.anchor.apply(&window);

            if let Some(margin) = config.margin_top {
                window.set_margin(gtk4_layer_shell::Edge::Top, margin);
            }
            if let Some(margin) = config.margin_bottom {
                window.set_margin(gtk4_layer_shell::Edge::Bottom, margin);
            }
            if let Some(margin) = config.margin_left {
                window.set_margin(gtk4_layer_shell::Edge::Left, margin);
            }
            if let Some(margin) = config.margin_right {
                window.set_margin(gtk4_layer_shell::Edge::Right, margin);
            }

            if matches!(config.layer, Layer::Background) {
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

            event_emitter
                .add_webview(config.label.clone(), SendWebView { webview })
                .await;

            windows.push(window);
        }

        Ok(windows)
    }
}
