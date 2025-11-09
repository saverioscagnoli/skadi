mod requirements;
mod vite;

use clap::Parser;
use common::config::Config;
use traccia::{LogLevel, debug, fatal};

#[derive(Debug, Clone, clap::Parser)]
#[clap(author, version, about)]
struct Args {
    #[arg(
        short,
        long,
        help = "Enable debug logging and webview inspector",
        default_value_t = false
    )]
    debug: bool,
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

fn init_logging(debug: bool) {
    let level = if debug || cfg!(debug_assertions) {
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

fn main() {
    let args = Args::parse();

    init_logging(args.debug);

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

    vite::init();

    if let Err(e) = app::setup_widgets(config) {
        fatal!("Application error: {}", e);
    }
}
