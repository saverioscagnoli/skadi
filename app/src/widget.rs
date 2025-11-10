use crate::window::Window;
use common::{
    config::{Anchor, Config, Layer, WidgetConfig},
    util::{self},
};
use gtk4::{
    gdk::{self, RGBA, prelude::MonitorExt},
    prelude::*,
};
use gtk4_layer_shell::{Edge, LayerShell};
use std::collections::HashMap;
use traccia::{debug, error, info};
use webkit6::{WebContext, WebView, prelude::WebViewExt};

pub struct WidgetFactory<'a> {
    app: &'a gtk4::Application,
    context: WebContext,
    monitors: HashMap<String, gdk::Monitor>,
}

impl<'a> WidgetFactory<'a> {
    pub fn new(app: &'a gtk4::Application) -> Self {
        let context = WebContext::new();

        let Some(display) = gdk::Display::default() else {
            error!("Failed to get default display for monitor enumeration.");

            return Self {
                app,
                context,
                monitors: HashMap::new(),
            };
        };

        let monitors = util::enumerate_monitors(&display);

        debug!(
            "Found {} monitor(s): {:?}",
            monitors.len(),
            monitors
                .values()
                .map(|m| m
                    .connector()
                    .map(|c| c.to_string())
                    .unwrap_or(String::from("Unknown monitor")))
                .collect::<Vec<_>>()
        );

        Self {
            app,
            context,
            monitors,
        }
    }

    pub fn create_widgets(&self, config: &Config, port: u16) -> Vec<Widget> {
        let mut widgets = Vec::new();

        for window_config in &config.widgets {
            let widget = Widget::new(
                self.app,
                window_config.clone(),
                &self.context,
                &self.monitors,
                port,
            );

            widgets.push(widget);
        }

        info!("Found {} widget(s)", widgets.len());

        widgets
    }
}

pub struct Widget {
    pub windows: Vec<Window>,
    config: WidgetConfig,
}

impl Widget {
    pub fn new(
        app: &gtk4::Application,
        config: WidgetConfig,
        context: &WebContext,
        monitors: &HashMap<String, gdk::Monitor>,
        port: u16,
    ) -> Self {
        let selected_monitors: Vec<&gdk::Monitor> =
            if config.monitors.iter().any(|m| m.to_lowercase() == "all") {
                monitors.values().collect()
            } else {
                config
                    .monitors
                    .iter()
                    .filter_map(|name| monitors.get(name))
                    .collect()
            };

        let mut windows = Vec::new();

        for (i, monitor) in selected_monitors.iter().enumerate() {
            let window = gtk4::ApplicationWindow::builder()
                .application(app)
                .title(&config.label)
                .resizable(false)
                .decorated(false)
                .build();

            let provider = gtk4::CssProvider::new();

            provider.load_from_data(&format!(
                r"
                window {{
                    background: rgba({}, {}, {}, {});
                }}
                ",
                config.background[0], config.background[1], config.background[2], config.opacity,
            ));

            debug!(
                "Injecting {:?} background color to {}",
                config.background, config.label
            );
            window
                .style_context()
                .add_provider(&provider, gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION);

            window.init_layer_shell();
            window.set_namespace(Some("wwwidgets"));

            let webview = WebView::builder()
                .web_context(context)
                .name(&config.label)
                .build();

            // On debug, enable web inspector
            if util::debug() {
                if let Some(settings) = webkit6::prelude::WebViewExt::settings(&webview) {
                    debug!("Web inspector enabled.");
                    settings.set_enable_developer_extras(true);
                }
            } else {
                webview.connect_context_menu(|_, _, _| true);
            }

            webview.set_background_color(&RGBA::TRANSPARENT);
            window.set_child(Some(&webview));

            webview.load_uri(&format!(
                "http://localhost:{}/html/{}.html",
                port, config.label
            ));

            let geometry = monitor.geometry();

            let width = config.width.as_pixel(geometry.width());
            let height = config.height.as_pixel(geometry.height());

            window.set_size_request(width, height);
            window.set_layer(config.layer.into());

            if config.exclusive {
                window.auto_exclusive_zone_enable();
            }

            config.anchor.apply(&window);

            let (top_margin, bottom_margin, left_margin, right_margin) = match config.anchor {
                Anchor::Left => (
                    config.margins.top,
                    config.margins.bottom,
                    config.margins.left + config.x,
                    config.margins.right,
                ),
                Anchor::Right => (
                    config.margins.top,
                    config.margins.bottom,
                    config.margins.left,
                    config.margins.right + config.x,
                ),
                Anchor::Top => (
                    config.margins.top + config.y,
                    config.margins.bottom,
                    config.margins.left,
                    config.margins.right,
                ),
                Anchor::Bottom => (
                    config.margins.top,
                    config.margins.bottom + config.y,
                    config.margins.left,
                    config.margins.right,
                ),
                Anchor::TopLeft => (
                    config.margins.top + config.y,
                    config.margins.bottom,
                    config.margins.left + config.x,
                    config.margins.right,
                ),
                Anchor::TopRight => (
                    config.margins.top + config.y,
                    config.margins.bottom,
                    config.margins.left,
                    config.margins.right + config.x,
                ),
                Anchor::TopCenter => (
                    config.margins.top + config.y,
                    config.margins.bottom,
                    config.margins.left + config.x,
                    config.margins.right,
                ),
                Anchor::BottomLeft => (
                    config.margins.top,
                    config.margins.bottom + config.y,
                    config.margins.left + config.x,
                    config.margins.right,
                ),
                Anchor::BottomRight => (
                    config.margins.top,
                    config.margins.bottom + config.y,
                    config.margins.left,
                    config.margins.right + config.x,
                ),
                Anchor::BottomCenter => (
                    config.margins.top,
                    config.margins.bottom + config.y,
                    config.margins.left + config.x,
                    config.margins.right,
                ),
            };

            window.set_monitor(Some(monitor));

            window.set_margin(Edge::Top, top_margin);
            window.set_margin(Edge::Right, right_margin);
            window.set_margin(Edge::Bottom, bottom_margin);
            window.set_margin(Edge::Left, left_margin);

            if config.layer == Layer::Background {
                // Set keyboard mode to none so it doesn't interfere with other windows
                window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);

                window.set_exclusive_zone(-1);
                window.set_anchor(gtk4_layer_shell::Edge::Top, true);
                window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
                window.set_anchor(gtk4_layer_shell::Edge::Left, true);
                window.set_anchor(gtk4_layer_shell::Edge::Right, true);
            }

            windows.push(Window {
                gtk_window: window,
                id: i as u32,
                monitor_id: monitor
                    .connector()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("Monitor {}", i)),
            });
        }

        Self { windows, config }
    }
}
