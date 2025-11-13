#![warn(clippy::use_self)]

mod requirements;
mod vite;

use clap::Parser;
use common::{config::Config, paths, util};
use traccia::{LogLevel, debug, fatal};

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
    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        if let Err(e) = app::start_server(
            config.port,
            paths::local_dir().join("build"),
            ready_tx,
            event_tx,
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

    if let Err(e) = app::setup_widgets(config, event_rx) {
        fatal!("Application error: {}", e);
    }
}
