mod registry;
mod requirements;
mod templates;
mod vite;

use crate::{requirements::Requirements, vite::ViteWorkspace};
use anyhow::Result;
use clap::Parser;
use common::{config::Config, paths};
use std::{
    path::PathBuf,
    process::{ExitStatus, Output, Stdio},
};

#[derive(Debug, Clone, Parser)]
pub enum Command {
    Generate,
}

#[derive(Debug, Clone, Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("Parsing configuration...");

    let config = match Config::parse() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    println!("Checking requirements...");

    let req = Requirements::new().await;

    if !req.check("node") {
        println!("Node.js is not installed. Attempting to install...");

        if let Err(e) = req.install_package("nodejs").await {
            eprintln!("Failed to install Node.js: {}", e);
            return;
        }

        println!("Node.js installed successfully.");
    } else {
        println!("Node.js is installed.");
    }

    if !req.check("npm") {
        println!("npm is not installed. Attempting to install...");

        if let Err(e) = req.install_package("npm").await {
            eprintln!("Failed to install npm: {}", e);
            return;
        }

        println!("npm installed successfully.");
    } else {
        println!("npm is installed.");
    }

    let Some(root) = paths::local() else {
        eprintln!("Failed to find the root path for the project.");
        return;
    };

    let vite = ViteWorkspace::new(root);

    if let Err(e) = vite.init(&config).await {
        eprintln!("Failed to initialize Vite workspace: {}", e);
        return;
    }

    match args.command {
        Command::Generate => {
            gtk::run(config);
        }
    }
}

pub async fn spawn_process<T: AsRef<str>>(
    cmd: T,
    args: &[&str],
    path: Option<&PathBuf>,
) -> Result<ExitStatus> {
    let path = path.map(|p| p.as_path());
    let mut builder = tokio::process::Command::new(cmd.as_ref());

    if let Some(path) = path {
        builder.current_dir(path);
    }

    builder.args(args);

    let status = builder.status().await?;

    Ok(status)
}

pub async fn spawn_process_quiet<T: AsRef<str>>(
    cmd: T,
    args: &[&str],
    path: Option<&PathBuf>,
) -> Result<ExitStatus> {
    let path = path.map(|p| p.as_path());
    let mut builder = tokio::process::Command::new(cmd.as_ref());

    if let Some(path) = path {
        builder.current_dir(path);
    }

    builder.args(args);
    builder.stdout(Stdio::null());
    builder.stderr(Stdio::null());

    let status = builder.status().await?;

    Ok(status)
}

pub async fn spawn_process_output<T: AsRef<str>>(
    cmd: T,
    args: &[&str],
    path: Option<&PathBuf>,
) -> Result<Output> {
    let path = path.map(|p| p.as_path());
    let mut builder = tokio::process::Command::new(cmd.as_ref());

    if let Some(path) = path {
        builder.current_dir(path);
    }

    builder.args(args);

    let output = builder.output().await?;

    Ok(output)
}
