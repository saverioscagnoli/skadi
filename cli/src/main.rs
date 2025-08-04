mod requirements;
mod templates;
mod vite;

use std::{path::PathBuf, sync::OnceLock};

use crate::{requirements::Requirements, vite::ViteWorkspace};
use clap::{ArgAction, Parser};
use common::config::Config;
use traccia::{Color, Colorize, LogLevel, Style, error, fatal, info, warn};

struct CustomFormatter;

impl traccia::Formatter for CustomFormatter {
    fn format(&self, record: &traccia::Record) -> String {
        let timestamp = chrono::Local::now()
            .format("%b %d %H:%M:%S")
            .to_string()
            .color(Color::Cyan)
            .dim();

        format!(
            "{} [{}] {}: {}",
            timestamp,
            record.target.dim(),
            record.level.default_coloring().to_lowercase(),
            record.message
        )
    }
}

fn log_level(debug: bool) -> LogLevel {
    if cfg!(debug_assertions) || debug {
        LogLevel::Debug
    } else {
        LogLevel::Info
    }
}

static DEBUG: OnceLock<bool> = OnceLock::new();
static DEV: OnceLock<bool> = OnceLock::new();

pub fn debug_mode() -> bool {
    cfg!(debug_assertions) || DEBUG.get().copied().unwrap_or(false)
}

pub fn dev() -> bool {
    DEV.get().copied().unwrap_or(false)
}

fn workspace_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let path = PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("skadi");

    Some(path)
}

#[derive(Debug, Parser)]
struct Args {
    /// Skip checking for requirements
    /// This will skip checking for node.js and npm
    #[arg(long, action = ArgAction::Set, default_value_t = false)]
    skip_requirements: bool,

    /// Enable development mode
    /// This will enable features like hot reloading and other development tools
    #[arg(long = "dev", action = ArgAction::SetTrue, default_value_t = false)]
    development: bool,

    /// Enable debug logging and other debug features like web inspector
    #[arg(long, action = ArgAction::SetTrue, default_value_t = false)]
    debug: bool,

    /// Path to the workspace directory
    /// This is where the frontend files will live and where the vite server will run
    #[arg(long)]
    workspace_dir: Option<String>,

    /// Shows the output of vite commands in the terminal
    #[arg(long, action = ArgAction::SetTrue, default_value_t = false)]
    show_output: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if DEBUG.set(args.debug).is_err() {
        eprintln!("Debug flag was already set, ignoring subsequent attempts.");
    }

    if DEV.set(args.development).is_err() {
        eprintln!("Development flag was already set, ignoring subsequent attempts.");
    }

    traccia::init_with_config(traccia::Config {
        level: log_level(args.debug),
        format: Some(Box::new(CustomFormatter)),
        ..Default::default()
    });

    if args.skip_requirements {
        warn!(
            "Skipping requirements check. This may lead to runtime errors if required binaries are missing."
        );
    } else {
        let req = Requirements::new().await;

        if let Err(e) = req.check_all().await {
            error!("Failed to check requirements: {}", e);
            error!("Please manually ensure that node.js and npm are installed.");
        }
    }

    let config = match Config::parse().await {
        Ok(c) => c,
        Err(e) => {
            fatal!("Failed to parse configuration: {}", e);
            return;
        }
    };

    info!("Using configuration {}", config.path().display());

    let Some(root) = args
        .workspace_dir
        .as_deref()
        .map(PathBuf::from)
        .or_else(workspace_path)
    else {
        fatal!("No workspace directory specified and no default found. Exiting.");
        return;
    };

    let vite = ViteWorkspace::new(root);

    if let Err(e) = vite.init(&args, &config).await {
        fatal!("Failed to initialize Vite workspace: {}", e);
        return;
    }

    gtk::run();
}
