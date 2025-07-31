use anyhow::{Result, anyhow};
use gtk4_layer_shell::LayerShell;
use serde::{
    Deserialize, Deserializer,
    de::{self, Visitor},
};
use std::{fmt, fs, path::PathBuf};

use crate::paths;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Framework {
    React,
    Vue,
    Svelte,
    Solid,
    Vanilla,
}

impl<'de> Deserialize<'de> for Framework {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "react" => Ok(Framework::React),
            "vue" => Ok(Framework::Vue),
            "svelte" => Ok(Framework::Svelte),
            "solid" => Ok(Framework::Solid),
            "vanilla" => Ok(Framework::Vanilla),
            _ => Err(serde::de::Error::custom(format!(
                "Unknown framework: {}",
                s
            ))),
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

    #[serde(default)]
    pub margin_top: Option<i32>,
    #[serde(default)]
    pub margin_bottom: Option<i32>,
    #[serde(default)]
    pub margin_left: Option<i32>,
    #[serde(default)]
    pub margin_right: Option<i32>,
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
        3499
    }

    pub fn default_layer() -> Layer {
        Layer::Top
    }

    pub fn default_exclusive() -> bool {
        false
    }

    pub fn path() -> PathBuf {
        paths::config()
            .map(|p| p.join("config.json"))
            .unwrap_or_else(|| PathBuf::from("config.json"))
    }

    pub fn parse() -> Result<Self> {
        let path = paths::config()
            .map(|p| p.join("config.json"))
            .ok_or_else(|| anyhow!("Could not find or create configuration directory"))?;

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                return Err(anyhow!(
                    "Failed to read config file {}: {}",
                    path.display(),
                    e
                ));
            }
        };

        let config = match serde_json::from_str(&content) {
            Ok(c) => c,
            Err(e) => {
                return Err(anyhow!(
                    "Failed to parse config file {}: {}",
                    path.display(),
                    e
                ));
            }
        };

        Ok(config)
    }
}
