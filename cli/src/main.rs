mod plugins;
mod requirements;
mod templates;
mod vite;

use clap::Parser;
use common::{config::Config, paths};
use traccia::{Color, Colorize, LogLevel, Style, fatal, warn};

use crate::{requirements::Requirements, vite::ViteWorkspace};

struct CustomFormatter;

impl traccia::Formatter for CustomFormatter {
    fn format(&self, record: &traccia::Record) -> String {
        let timestamp = chrono::Local::now().format("%b %d %H:%M:%S").to_string();

        format!(
            "{} [{}] {}: {}",
            timestamp.color(Color::Cyan).dim(),
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
enum Command {
    Generate,

    Dev,
}

#[derive(Debug, Clone, Parser)]
struct Args {
    /// Skip requirements checks
    #[clap(long, default_value_t = false)]
    skip_checks: bool,

    #[clap(subcommand)]
    command: Command,
}

// #[tokio::main]
// async fn main() {
//     traccia::init_with_config(traccia::Config {
//         level: log_level(),
//         format: Some(Box::new(CustomFormatter)),
//         ..Default::default()
//     });

//     let args = Args::parse();

//     let config = match Config::parse() {
//         Ok(c) => c,
//         Err(e) => {
//             fatal!("Failed to parse configuration: {}", e);
//             return;
//         }
//     };

//     match args.command {
//         Command::Init { skip_checks } => {
//             if !skip_checks {
//                 requirements::node_check().await;
//                 requirements::yarn_check().await;
//             } else {
//                 warn!("Skipping requirements checks as requested.");
//             }

//             if let Err(e) = vite::init_workspace().await {
//                 fatal!("Failed to initialize workspace: {}", e);
//                 return;
//             }
//         }

//         Command::Generate => {
//             if let Err(e) = vite::generate(&config).await {
//                 fatal!("Failed to generate workspace: {}", e);
//                 return;
//             }
//         }

//         Command::Dev => {
//             if let Err(e) = vite::run_dev().await {
//                 fatal!("Failed to run development server: {}", e);
//                 return;
//             }
//         }
//     }
// }

#[tokio::main]
async fn main() {
    traccia::init_with_config(traccia::Config {
        level: log_level(),
        format: Some(Box::new(CustomFormatter)),
        ..Default::default()
    });

    let args = Args::parse();

    if !args.skip_checks
        && let Err(e) = Requirements::check().await
    {
        fatal!("Requirements check failed: {}", e);
        return;
    }

    let config = match Config::parse() {
        Ok(c) => c,
        Err(e) => {
            fatal!("Failed to parse configuration: {}", e);
            return;
        }
    };

    let Some(local) = paths::local() else {
        fatal!("Failed to find or create local directory `~/.local/share/skadi`");
        return;
    };

    let Some(plugins) = paths::plugins() else {
        fatal!("Failed to find or create plugins directory");
        return;
    };

    let mut vite = ViteWorkspace::init(&local, &plugins);

    match args.command {
        Command::Generate => {
            if let Err(e) = vite.generate(&config).await {
                fatal!("{}", e);
                return;
            }
        }

        _ => {}
    }
}
