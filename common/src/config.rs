use crate::templates::Templates;
use core::fmt;
use gtk4_layer_shell::LayerShell;
use serde::{
    Deserialize, Deserializer,
    de::{self, Visitor},
};
use std::{error::Error, path::PathBuf};
use traccia::{debug, info};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    Pixel(i32),
    Percentage(f32),
}

impl Dimension {
    pub fn as_pixel(&self, total: i32) -> i32 {
        match self {
            Self::Pixel(p) => *p,
            Self::Percentage(p) => (total as f32 * p / 100.0).round() as i32,
        }
    }
}

impl<'de> Deserialize<'de> for Dimension {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DimensionVisitor;

        impl<'de> Visitor<'de> for DimensionVisitor {
            type Value = Dimension;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string like \"50%\" or \"100\", or a number")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if s.ends_with('%') {
                    let percent_str = s.trim_end_matches('%');
                    let percent: f32 = percent_str
                        .parse()
                        .map_err(|_| E::custom(format!("Invalid percentage: {}", s)))?;
                    Ok(Dimension::Percentage(percent))
                } else {
                    let pixels: i32 = s
                        .parse()
                        .map_err(|_| E::custom(format!("Invalid pixel value: {}", s)))?;
                    Ok(Dimension::Pixel(pixels))
                }
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Dimension::Pixel(value as i32))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if value < 0 {
                    return Err(E::custom("pixel value cannot be negative"));
                }
                Ok(Dimension::Pixel(value as i32))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Dimension::Pixel(value.round() as i32))
            }
        }

        deserializer.deserialize_any(DimensionVisitor)
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Anchor {
    Top,
    Left,
    Right,
    Bottom,

    #[serde(alias = "top-left", alias = "top left")]
    TopLeft,

    #[serde(alias = "top-right", alias = "top right")]
    TopRight,

    #[serde(alias = "top-center", alias = "top center")]
    TopCenter,

    #[serde(alias = "bottom-left", alias = "bottom left")]
    BottomLeft,

    #[serde(alias = "bottom-right", alias = "bottom right")]
    BottomRight,

    #[serde(alias = "bottom-center", alias = "bottom center")]
    BottomCenter,
}

impl Anchor {
    pub fn apply(&self, window: &gtk4::ApplicationWindow) {
        match self {
            Anchor::Top => window.set_anchor(gtk4_layer_shell::Edge::Top, true),
            Anchor::Left => window.set_anchor(gtk4_layer_shell::Edge::Left, true),
            Anchor::Right => window.set_anchor(gtk4_layer_shell::Edge::Right, true),
            Anchor::Bottom => window.set_anchor(gtk4_layer_shell::Edge::Bottom, true),
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
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Layer {
    #[default]
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

#[derive(Debug, Default, Clone, Deserialize)]
pub struct Margins {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WidgetConfig {
    pub monitors: Vec<String>,
    pub label: String,
    pub width: Dimension,
    pub height: Dimension,
    pub x: i32,
    pub y: i32,
    pub anchor: Anchor,

    #[serde(default)]
    pub layer: Layer,

    #[serde(default)]
    pub exclusive: bool,

    #[serde(default)]
    pub margins: Margins,

    /// Path to the widget's index file (the file that is the 'parent' of all your components)
    /// If you were coding in a normal react app, that would be your App.jsx.
    pub index: PathBuf,

    #[serde(default)]
    pub background: [u8; 3],

    #[serde(default = "Config::default_opacity")]
    pub opacity: f64,

    /// Whether the widget should be hidden by default
    #[serde(default)]
    pub hidden: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "Config::default_port")]
    pub port: u16,
    pub widgets: Vec<WidgetConfig>,
}

impl Config {
    fn default_port() -> u16 {
        10978
    }

    fn default_opacity() -> f64 {
        1.0
    }

    pub fn parse() -> Result<Self, Box<dyn Error>> {
        let Some(config_path) = dirs::config_dir().map(|p| p.join("wwwidgets").join("config.json"))
        else {
            return Err("Could not determine config directory".into());
        };

        if !config_path.exists() {
            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            debug!("Configuration file wasn't found. Creating a default one");
            std::fs::write(&config_path, Templates::DEFAULT_CONFIG)?;
        }

        let content = std::fs::read_to_string(&config_path)?;
        let config: Self = serde_json::from_str(&content)?;

        info!("Using configuration file at {:?}", config_path);
        debug!("{:?}", config);
        Ok(config)
    }
}
