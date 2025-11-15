use common::util;
use std::error::Error;
use traccia::{Colorize, Style, info, warn};
use which::which;

const NODEJS_SCRIPT: &str = r#"
    # Download and install nvm
    curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.3/install.sh | bash

    # Source nvm
    . "$HOME/.nvm/nvm.sh"

    # Download and install Node.js
    nvm install 24
"#;

#[derive(Debug)]
enum Distro {
    Debian,  // apt install <package>
    Ubuntu,  // apt install <package>
    Fedora,  // dnf install <package>
    Arch,    // pacman -S <package>
    Manjaro, // pacman -S <package>
    Mint,    // apt install <package>
    PopOS,   // apt install <package>
    Other,   // Use nvm
}

impl Distro {
    fn detect() -> Self {
        // Try to read /etc/os-release first (most modern distros)
        if let Ok(contents) = std::fs::read_to_string("/etc/os-release") {
            let lower = contents.to_lowercase();

            if lower.contains("id=ubuntu") || lower.contains("id=\"ubuntu\"") {
                return Self::Ubuntu;
            } else if lower.contains("id=debian") || lower.contains("id=\"debian\"") {
                return Self::Debian;
            } else if lower.contains("id=fedora") || lower.contains("id=\"fedora\"") {
                return Self::Fedora;
            } else if lower.contains("id=arch") || lower.contains("id=\"arch\"") {
                return Self::Arch;
            } else if lower.contains("id=manjaro") || lower.contains("id=\"manjaro\"") {
                return Self::Manjaro;
            } else if lower.contains("id=linuxmint") || lower.contains("id=\"linuxmint\"") {
                return Self::Mint;
            } else if lower.contains("id=pop") || lower.contains("id=\"pop\"") {
                return Self::PopOS;
            }
        }

        // Fallback: check for specific release files
        if std::fs::metadata("/etc/arch-release").is_ok() {
            return Self::Arch;
        } else if std::fs::metadata("/etc/fedora-release").is_ok() {
            return Self::Fedora;
        } else if std::fs::metadata("/etc/debian_version").is_ok() {
            return Self::Debian;
        }

        Self::Other
    }

    fn install_node(&self) -> &str {
        match self {
            Self::Debian => "sudo apt install nodejs -y",
            Self::Ubuntu => "sudo apt install nodejs -y",
            Self::Fedora => "sudo dnf install nodejs -y",
            Self::Arch => "sudo pacman -S nodejs --noconfirm",
            Self::Manjaro => "sudo pacman -S nodejs --noconfirm",
            Self::Mint => "sudo apt install nodejs -y",
            Self::PopOS => "sudo apt install nodejs -y",
            Self::Other => NODEJS_SCRIPT,
        }
    }
}

fn install_node(distro: &Distro) -> Result<(), Box<dyn Error>> {
    util::spawn_capture(distro.install_node(), |l| {
        println!("{}", l.dim());
    })?;

    Ok(())
}

fn install_yarn() -> Result<(), Box<dyn Error>> {
    util::spawn_capture("sudo npm i -g yarn", |l| {
        println!("{}", l.dim());
    })?;

    Ok(())
}

/// Requirements checks
///
/// If the required programs are not intalled,
/// the user will have the option to install them automatically.
///
/// 1) Node.js
/// 2) Yarn
///    TODO: Ask for things like gtk4, gtk-layer-shell, etc. too
pub fn check() -> Result<(), Box<dyn Error>> {
    let distro = Distro::detect();

    // This flag is for when both node.js are not installed
    // 1) user says yes to node.js
    // 2) it's assumed that if they say yes they also want yarn so check will be skipped
    // Also, when nodejs is installed but yarn is not, it will ask if they want to install yarn,
    // because default false so it will check if !answered_yes
    let mut answered_yes = false;

    // Check node
    if which("node").is_err() {
        // First ask if user wants to install nodejs first
        answered_yes = match util::ask_yes_no(|| {
            let message = if let Distro::Other = distro {
                String::from("Node.js is not available. Do you want to install it via NVM?")
            } else {
                format!(
                    "Node.js is not available. Do you wish to install it with {}?",
                    distro
                        .install_node()
                        .color(traccia::Color::BrightGreen)
                        .bold()
                )
            };

            warn!("{} (y/n)", message);
        }) {
            Ok(a) => a,
            Err(_) => {
                return Err("Please install Node.js and Yarn before running wwwidgets.".into());
            }
        };

        if !answered_yes {
            return Err("Please install both Node.js and Yarn before running wwwidgets.".into());
        }

        install_node(&distro)?;
        info!("Node.js was installed.");
    }

    // Check yarn
    if which("yarn").is_err() {
        if answered_yes {
            warn!("Installing Yarn...");
        } else {
            let answer = match util::ask_yes_no(|| {
                warn!(
                    "Yarn is not available. do you wish to install it with {}? (y/n)",
                    "sudo npm i -g yarn"
                        .color(traccia::Color::BrightGreen)
                        .bold()
                );
            }) {
                Ok(a) => a,
                Err(_) => {
                    return Err(
                        "Please install both Node.js  Yarn before running wwwidgets.".into(),
                    );
                }
            };

            if !answer {
                return Err(
                    "Please install both Node.js and Yarn before running wwwidgets.".into(),
                );
            }
        }

        install_yarn()?;
        info!("Yarn was installed.");
    }

    Ok(())
}
