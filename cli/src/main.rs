mod registry;
mod requirements;
mod spinner;
mod templates;
mod vite;

use crate::{requirements::Requirements, spinner::Spinner, vite::ViteWorkspace};
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

    let mut spinner = Spinner::new()
        .with_message("Parsing configuration...")
        .with_delay(std::time::Duration::from_millis(100))
        .start();

    let config = match Config::parse() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    spinner.update_message("Checking requirements...");

    let req = Requirements::new().await;
    let mut root_granted = false;

    if !req.check("node") {
        if !req.check_root() {
            spinner.finish_with_symbol_and_message(
                "⚠️",
                format!(
                    "Please enter root password to install Node.js. This command will run:\n{}",
                    req.distro.install_command("nodejs")
                )
                .as_str(),
            );

            if let Ok(status) = spawn_process_quiet("sudo", &["-v"], None).await {
                if status.success() {
                    root_granted = true;
                } else {
                    eprintln!("Failed to run sudo command.");
                    return;
                }
            } else {
                eprintln!("Failed to run sudo command.");
                return;
            }

            print!("\x1B[1A\x1B[2K");
            spinner = Spinner::new()
                .with_message("Installing Node.js...")
                .with_delay(std::time::Duration::from_millis(100))
                .start();
        }

        if let Err(e) = req.install_package("nodejs").await {
            eprintln!("Failed to install Node.js: {}", e);
            return;
        }

        spinner.update_message("Node.js installed successfully.");
    } else {
        spinner.update_message("Node.js is installed.");
    }

    if !req.check("npm") {
        if !root_granted && !req.check_root() {
            spinner.finish_with_symbol_and_message(
                "⚠️",
                format!(
                    "Please enter root password to install Node.js. This command will run:\n{}",
                    req.distro.install_command("npm")
                )
                .as_str(),
            );

            spawn_process_quiet("sudo", &["-v"], None).await.ok();
        }

        print!("\x1B[1A\x1B[2K");
        spinner = Spinner::new()
            .with_message("Installing Node.js...")
            .with_delay(std::time::Duration::from_millis(100))
            .start();

        if let Err(e) = req.install_package("npm").await {
            eprintln!("Failed to install npm: {}", e);
            return;
        }

        spinner.update_message("npm installed successfully.");
    } else {
        spinner.update_message("npm is installed.");
    }

    let Some(root) = paths::local() else {
        eprintln!("Failed to find the root path for the project.");
        return;
    };

    let vite = ViteWorkspace::new(root);

    if let Err(e) = vite.init(&config, spinner).await {
        eprintln!("Failed to initialize Vite workspace: {}", e);
        return;
    }

    match args.command {
        Command::Generate => {
            gtk::run(config);
        }
    }
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
