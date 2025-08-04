mod requirements;
mod templates;
mod vite;

use crate::{requirements::Requirements, vite::ViteWorkspace};
use clap::{ArgAction, Parser};
use common::{config::Config, debug_mode, set_debug_mode, set_dev_mode};
use std::path::PathBuf;
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

fn log_level() -> LogLevel {
    if debug_mode() {
        LogLevel::Debug
    } else {
        LogLevel::Info
    }
}

fn workspace_path() -> PathBuf {
    let home = std::env::var("HOME").expect("How do you not have a HOME?");
    let path = PathBuf::from(home)
        .join(".local")
        .join("share")
        .join("skadi");

    return path;
}

#[derive(Debug, Parser)]
struct Args {
    /// Skip checking for requirements.
    #[arg(long, action = ArgAction::SetTrue, default_value_t = false)]
    skip_requirements: bool,

    /// Skip building the Vite project.
    /// This will skip npm install and such
    #[arg(long, action = ArgAction::SetTrue, default_value_t = false, conflicts_with = "development")]
    skip_vite: bool,

    /// Enable development mode.
    /// This will run the Vite dev server instead of building for production
    #[arg(long = "dev", action = ArgAction::SetTrue, default_value_t = false)]
    development: bool,

    /// Enable debug logging and other debug features like web inspector
    #[arg(long, action = ArgAction::SetTrue, default_value_t = false)]
    debug: bool,

    /// Path to the workspace directory.
    /// This is where the frontend files will live and where the vite server will run
    #[arg(long, default_value_t = workspace_path().to_string_lossy().to_string(), conflicts_with = "skip_vite")]
    workspace_dir: String,

    /// Shows the output of vite commands in the terminal
    #[arg(long, action = ArgAction::SetTrue, default_value_t = false)]
    show_output: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    set_debug_mode(args.debug);
    set_dev_mode(args.development);

    traccia::init_with_config(traccia::Config {
        level: log_level(),
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

    let workspace_path = PathBuf::from(&args.workspace_dir);

    if args.skip_vite {
        warn!("Skipping vite initialization. Ensure that the project was built previously.");
    } else {
        let vite = ViteWorkspace::new(&workspace_path);

        if let Err(e) = vite.init(&config, &args).await {
            fatal!("Failed to initialize Vite workspace: {}", e);
            return;
        }
    }

    gtk::run(&config, &workspace_path);
}
