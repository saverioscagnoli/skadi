#![warn(clippy::use_self)]

mod requirements;
mod vite;

use clap::{Parser, Subcommand};
use common::{config::Config, paths, util};
use traccia::{LogLevel, debug, fatal, info};

#[derive(Debug, Clone, clap::Parser)]
#[clap(author, version, about)]
struct Args {
    #[arg(
        long,
        help = "Enable debug logging and webview inspector",
        default_value_t = false
    )]
    debug: bool,

    #[arg(long, help = "Run in development mode", default_value_t = false)]
    dev: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Clone, Subcommand)]
enum Commands {
    /// Clear the build cache and temporary files
    Clean,
}

struct LogFormatter;

impl traccia::Formatter for LogFormatter {
    fn format(&self, record: &traccia::Record) -> String {
        format!(
            "{} {}",
            record.level.default_coloring().to_lowercase(),
            record.message
        )
    }
}

fn init_logging() {
    let level = if util::debug() || cfg!(debug_assertions) {
        LogLevel::Debug
    } else {
        LogLevel::Info
    };

    traccia::init_with_config(traccia::Config {
        level,
        targets: vec![Box::new(traccia::Console::new())],
        format: Some(Box::new(LogFormatter)),
    });
}

fn clean_cache() {
    let local_dir = paths::local_dir();

    info!("Cleaning build cache at {}", local_dir.display());

    if local_dir.exists() {
        match std::fs::remove_dir_all(&local_dir) {
            Ok(_) => {
                info!("Successfully cleaned build cache");

                // Recreate the directory
                if let Err(e) = std::fs::create_dir_all(&local_dir) {
                    fatal!("Failed to recreate local directory: {}", e);
                }
            }
            Err(e) => {
                fatal!("Failed to clean build cache: {}", e);
            }
        }
    } else {
        info!("Build cache directory doesn't exist, nothing to clean");
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.debug || cfg!(debug_assertions) {
        util::set_debug(true);
    }

    if args.dev {
        util::set_debug(true);
        util::set_dev(true);
    }

    init_logging();

    // Handle subcommands
    if let Some(command) = args.command {
        match command {
            Commands::Clean => {
                clean_cache();
                return;
            }
        }
    }

    debug!("Checking requirements...");

    if let Err(e) = requirements::check() {
        fatal!("{}", e);
        return;
    }

    debug!("All requirements are met.");

    let config = match Config::parse() {
        Ok(c) => c,
        Err(e) => {
            fatal!("{}", e);
            return;
        }
    };

    if let Err(e) = vite::init(&config) {
        fatal!("Vite init failed: {}", e);
        return;
    }

    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel::<()>();
    let (window_tx, window_rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        if let Err(e) = app::start_server(
            config.port,
            paths::local_dir().join("build"),
            ready_tx,
            window_tx,
        )
        .await
        {
            fatal!("{}", e);
            std::process::exit(1);
        }
    });

    if let Err(e) = ready_rx.await {
        fatal!("Failed to start server: {}", e);
        return;
    }

    if let Err(e) = app::setup_widgets(config, window_rx) {
        fatal!("Application error: {}", e);
    }
}
