use crate::error::{self, SkadiError};
use gtk4::{
    gdk::{
        prelude::{DisplayExt, MonitorExt},
        Display, Monitor, RGBA,
    },
    gio::prelude::ListModelExtManual,
    glib::object::ObjectExt,
    prelude::{GtkWindowExt, WidgetExt},
};
use gtk4_layer_shell::LayerShell;

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer,
};
use std::{fmt, fs, path::PathBuf};
use traccia::{error, info};
use webkit6::prelude::WebViewExt;

struct Paths;

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

#[derive(Debug, Clone)]
pub enum Anchor {
    Top,
    Left,
    Right,
    Bottom,
    Center,
    TopLeft,
    TopRight,
    TopCenter,
    BottomLeft,
    BottomRight,
    BottomCenter,
}

impl<'de> Deserialize<'de> for Anchor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AnchorVisitor;

        impl<'de> Visitor<'de> for AnchorVisitor {
            type Value = Anchor;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("an anchor string like 'top', 'left', 'top center', etc.")
            }

            fn visit_str<E>(self, value: &str) -> Result<Anchor, E>
            where
                E: de::Error,
            {
                match value.to_lowercase().as_str() {
                    "top" => Ok(Anchor::Top),
                    "left" => Ok(Anchor::Left),
                    "right" => Ok(Anchor::Right),
                    "bottom" => Ok(Anchor::Bottom),
                    "center" => Ok(Anchor::Center),
                    "top left" | "topleft" => Ok(Anchor::TopLeft),
                    "top right" | "topright" => Ok(Anchor::TopRight),
                    "top center" | "topcenter" => Ok(Anchor::TopCenter),
                    "bottom left" | "bottomleft" => Ok(Anchor::BottomLeft),
                    "bottom right" | "bottomright" => Ok(Anchor::BottomRight),
                    "bottom center" | "bottomcenter" => Ok(Anchor::BottomCenter),
                    _ => Err(E::custom(format!("unknown anchor: {}", value))),
                }
            }
        }

        deserializer.deserialize_str(AnchorVisitor)
    }
}

impl Anchor {
    pub fn apply(&self, window: &gtk4::ApplicationWindow) {
        match self {
            Anchor::Top => window.set_anchor(gtk4_layer_shell::Edge::Top, true),
            Anchor::Left => window.set_anchor(gtk4_layer_shell::Edge::Left, true),
            Anchor::Right => window.set_anchor(gtk4_layer_shell::Edge::Right, true),
            Anchor::Bottom => window.set_anchor(gtk4_layer_shell::Edge::Bottom, true),
            Anchor::Center => {
                window.set_anchor(gtk4_layer_shell::Edge::Top, false);
                window.set_anchor(gtk4_layer_shell::Edge::Left, false);
                window.set_anchor(gtk4_layer_shell::Edge::Right, false);
                window.set_anchor(gtk4_layer_shell::Edge::Bottom, false);
            }
            Anchor::TopLeft => {
                window.set_anchor(gtk4_layer_shell::Edge::Top, true);
                window.set_anchor(gtk4_layer_shell::Edge::Left, true);
            }
            Anchor::TopRight => {
                window.set_anchor(gtk4_layer_shell::Edge::Top, true);
                window.set_anchor(gtk4_layer_shell::Edge::Right, true);
            }
            Anchor::TopCenter => {
                window.set_anchor(gtk4_layer_shell::Edge::Top, true);
                window.set_anchor(gtk4_layer_shell::Edge::Left, false);
                window.set_anchor(gtk4_layer_shell::Edge::Right, false);
            }
            Anchor::BottomLeft => {
                window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
                window.set_anchor(gtk4_layer_shell::Edge::Left, true);
            }
            Anchor::BottomRight => {
                window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
                window.set_anchor(gtk4_layer_shell::Edge::Right, true);
            }
            Anchor::BottomCenter => {
                window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
                window.set_anchor(gtk4_layer_shell::Edge::Left, false);
                window.set_anchor(gtk4_layer_shell::Edge::Right, false);
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Dimension {
    Pixel(i32),
    Percentage(f32),
}

impl Dimension {
    pub fn as_pixel(&self, total: i32) -> i32 {
        match self {
            Dimension::Pixel(p) => *p,
            Dimension::Percentage(p) => (total as f32 * p / 100.0).round() as i32,
        }
    }
}

impl<'de> Deserialize<'de> for Dimension {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        if s.ends_with('%') {
            let percent_str = &s[..s.len() - 1];
            let percent: f32 = percent_str
                .parse()
                .map_err(|_| serde::de::Error::custom(format!("Invalid percentage: {}", s)))?;
            Ok(Dimension::Percentage(percent))
        } else {
            let pixels: i32 = s
                .parse()
                .map_err(|_| serde::de::Error::custom(format!("Invalid pixel value: {}", s)))?;
            Ok(Dimension::Pixel(pixels))
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Layer {
    Top,
    Bottom,
    Background,
    Overlay,
}

impl From<Layer> for gtk4_layer_shell::Layer {
    fn from(layer: Layer) -> Self {
        match layer {
            Layer::Top => gtk4_layer_shell::Layer::Top,
            Layer::Bottom => gtk4_layer_shell::Layer::Bottom,
            Layer::Background => gtk4_layer_shell::Layer::Background,
            Layer::Overlay => gtk4_layer_shell::Layer::Overlay,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WindowConfig {
    pub monitor: String,
    pub label: String,
    pub width: Dimension,
    pub height: Dimension,
    pub anchor: Anchor,

    #[serde(default = "Config::default_layer")]
    pub layer: Layer,

    #[serde(default = "Config::default_exclusive")]
    pub exclusive: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app_id: String,
    #[serde(default = "Config::default_port")]
    pub port: u16,
    pub windows: Vec<WindowConfig>,
}

impl Config {
    pub fn default_port() -> u16 {
        3497
    }

    pub fn default_layer() -> Layer {
        Layer::Top
    }

    pub fn default_exclusive() -> bool {
        false
    }

    pub fn parse() -> Result<Self, SkadiError> {
        let paths = Paths::possible_configs()?;

        for path in &paths {
            if path.exists() {
                let content = fs::read_to_string(&path)?;
                let value = jsonc_parser::parse_to_serde_value(
                    &content,
                    &jsonc_parser::ParseOptions::default(),
                )
                .map_err(|e| SkadiError::ConfigParsing(path.clone(), e.to_string()))?
                .ok_or_else(|| {
                    SkadiError::ConfigParsing(path.clone(), "No value returned".to_string())
                })?;

                let config: Config = serde_json::from_value(value)
                    .map_err(|e| SkadiError::ConfigParsing(path.clone(), e.to_string()))?;

                info!("Loaded configuration from {}", path.display());
                info!("App ID: {}", config.app_id);

                return Ok(config);
            }
        }

        Err(SkadiError::ConfigNotSpecified(paths))
    }

    pub fn create_app(&self) -> gtk4::Application {
        gtk4::Application::builder()
            .application_id(&self.app_id)
            .build()
    }

    pub fn setup_windows(
        &self,
        app: &gtk4::Application,
    ) -> Result<Vec<gtk4::ApplicationWindow>, SkadiError> {
        let mut windows = Vec::new();

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
            let uri = format!("http://localhost:{}", self.port);

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

            let Some(display) = Display::default() else {
                error!("Failed to get default display");
                continue;
            };

            let monitors = display.monitors();

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

            window.set_resizable(false);
            window.set_decorated(false);

            windows.push(window);
        }

        Ok(windows)
    }
}
