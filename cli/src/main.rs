mod requirements;
mod vite;

use clap::Parser;
use traccia::{Color, Colorize, LogLevel, Style};

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
    if cfg!(debug_assertions) {
        LogLevel::Debug
    } else {
        LogLevel::Info
    }
}

#[derive(Debug, Clone, Parser)]
struct Args {
    /// Skip requirements checks
    #[clap(long, default_value_t = false)]
    skip_checks: bool,
}

#[tokio::main]
async fn main() {
    traccia::init_with_config(traccia::Config {
        level: log_level(),
        format: Some(Box::new(CustomFormatter)),
        ..Default::default()
    });

    let args = Args::parse();

    if !args.skip_checks {
        requirements::node_check().await;
        requirements::yarn_check().await;
    }

    vite::init_workspace().await;
}
