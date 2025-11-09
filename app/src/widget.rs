use crate::window::Window;
use common::{
    config::{Config, Layer, WidgetConfig},
    util::enumerate_monitors,
};
use gtk4::{
    gdk::{self, RGBA, prelude::MonitorExt},
    prelude::{GtkWindowExt, WidgetExt},
};
use gtk4_layer_shell::{Edge, LayerShell};
use std::collections::HashMap;
use traccia::{debug, error};
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

        let monitors = enumerate_monitors(&display);

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

    pub fn create_widgets(&self, config: &Config) -> Vec<Widget> {
        let mut widgets = Vec::new();

        for window_config in &config.widgets {
            let widget = Widget::new(
                self.app,
                window_config.clone(),
                &self.context,
                &self.monitors,
            );

            widgets.push(widget);
        }

        debug!("Found {} widget(s)", widgets.len());

        widgets
    }
}

pub struct Widget {
    pub windows: Vec<Window>,
    context: WebContext,
    config: WidgetConfig,
}

impl Widget {
    pub fn new(
        app: &gtk4::Application,
        config: WidgetConfig,
        context: &WebContext,
        monitors: &HashMap<String, gdk::Monitor>,
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

            let webview = WebView::builder()
                .web_context(context)
                .name(&config.label)
                .build();

            webview.set_background_color(&RGBA::TRANSPARENT);
            window.set_child(Some(&webview));

            let geometry = monitor.geometry();

            let width = config.width.as_pixel(geometry.width());
            let height = config.height.as_pixel(geometry.height());

            window.set_width_request(width);
            window.set_height_request(height);

            window.init_layer_shell();
            window.set_layer(config.layer.into());

            if config.exclusive {
                window.auto_exclusive_zone_enable();
            }

            if config.x != 0 || config.y != 0 {
                todo!("Positioning windows is not yet implemented.");
            }

            config.anchor.apply(&window);

            window.set_monitor(Some(&monitor));

            window.set_margin(Edge::Top, config.margins.top);
            window.set_margin(Edge::Right, config.margins.right);
            window.set_margin(Edge::Bottom, config.margins.bottom);
            window.set_margin(Edge::Left, config.margins.left);

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

        Self {
            windows,
            config,
            context: context.clone(),
        }
    }
}
