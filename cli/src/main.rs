mod requirements;
mod vite;

use std::sync::OnceLock;

use clap::{ArgAction, Parser};
use traccia::{Color, Colorize, LogLevel, Style, error, warn};

use crate::requirements::Requirements;

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

pub fn debug() -> bool {
    *DEBUG.get().unwrap_or(&false)
}

#[derive(Debug, Parser)]
struct Args {
    /// Skip checking for requirements
    /// This will skip checking for node.js and npm
    #[arg(long, action = ArgAction::Set, default_value_t = false)]
    skip_requirements: bool,

    /// Enable debug logging and other debug features like web inspector
    #[arg(long, action = ArgAction::SetTrue, default_value_t = false)]
    debug: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if DEBUG.set(args.debug).is_err() {
        eprintln!("Debug flag was already set, ignoring subsequent attempts.");
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

    gtk::run();
}
