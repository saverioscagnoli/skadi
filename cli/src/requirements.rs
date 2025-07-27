use std::sync::LazyLock;
use traccia::{fatal, info, warn};

enum Distro {
    Ubuntu,
    Fedora,
    Arch,
    Unknown,
}

impl Distro {
    fn detect() -> Self {
        let os_release = std::fs::read_to_string("/etc/os-release").unwrap_or_default();

        for line in os_release.lines() {
            if line.starts_with("ID=") {
                let id = line.split('=').nth(1).unwrap_or("").trim_matches('"');

                return match id {
                    "ubuntu" => Distro::Ubuntu,
                    "fedora" => Distro::Fedora,
                    "arch" => Distro::Arch,
                    _ => Distro::Unknown,
                };
            }
        }

        Self::Unknown
    }
}

static DISTRO: LazyLock<Distro> = LazyLock::new(Distro::detect);

async fn node_installed() -> bool {
    let output = tokio::process::Command::new("node")
        .arg("--version")
        .output()
        .await;

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

async fn npm_installed() -> bool {
    let output = tokio::process::Command::new("npm")
        .arg("--version")
        .output()
        .await;

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

async fn yarn_installed() -> bool {
    let output = tokio::process::Command::new("yarn")
        .arg("--version")
        .output()
        .await;

    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

pub async fn node_check() {
    warn!("Checking for Node.js installation...");

    if !node_installed().await {
        warn!("Node.js is not installed. Attempting to install...");

        let result;

        match *DISTRO {
            Distro::Ubuntu => {
                result = tokio::process::Command::new("sudo")
                    .arg("apt-get")
                    .arg("install")
                    .arg("-y")
                    .arg("nodejs")
                    .output()
                    .await;
            }

            Distro::Fedora => {
                result = tokio::process::Command::new("sudo")
                    .arg("dnf")
                    .arg("install")
                    .arg("-y")
                    .arg("nodejs")
                    .output()
                    .await;
            }

            Distro::Arch => {
                result = tokio::process::Command::new("sudo")
                    .arg("pacman")
                    .arg("-S")
                    .arg("--noconfirm")
                    .arg("nodejs")
                    .output()
                    .await;
            }

            Distro::Unknown => {
                fatal!("Unsupported Linux distribution. Please install Node.js manually.");
                return;
            }
        }

        let result = match result {
            Ok(o) => o,
            Err(e) => {
                fatal!("Failed to install Node.js: {}", e);
                return;
            }
        };

        if !result.status.success() {
            fatal!(
                "Failed to install Node.js: {}",
                String::from_utf8_lossy(&result.stderr)
            );
            return;
        }

        info!("Node.js installed successfully.");
        npm_check().await;
    } else {
        info!("Node.js is installed.");
    }
}

async fn npm_check() {
    warn!("Checking for NPM installation...");

    if !npm_installed().await {
        warn!("NPM is not installed. Attempting to install...");

        let result;

        match *DISTRO {
            Distro::Ubuntu => {
                result = tokio::process::Command::new("sudo")
                    .arg("apt-get")
                    .arg("install")
                    .arg("-y")
                    .arg("npm")
                    .output()
                    .await;
            }

            Distro::Fedora => {
                result = tokio::process::Command::new("sudo")
                    .arg("dnf")
                    .arg("install")
                    .arg("-y")
                    .arg("npm")
                    .output()
                    .await;
            }

            Distro::Arch => {
                result = tokio::process::Command::new("sudo")
                    .arg("pacman")
                    .arg("-S")
                    .arg("--noconfirm")
                    .arg("npm")
                    .output()
                    .await;
            }

            Distro::Unknown => {
                fatal!("Unsupported Linux distribution. Please install NPM manually.");
                return;
            }
        }

        let result = match result {
            Ok(o) => o,
            Err(e) => {
                fatal!("Failed to install NPM: {}", e);
                return;
            }
        };

        if !result.status.success() {
            fatal!(
                "Failed to install NPM: {}",
                String::from_utf8_lossy(&result.stderr)
            );
            return;
        }

        info!("NPM installed successfully.");
    } else {
        info!("NPM is installed.");
    }
}

pub async fn yarn_check() {
    warn!("Checking for Yarn installation...");

    if !yarn_installed().await {
        warn!("Yarn is not installed. Attempting to install...");

        if !npm_installed().await {
            warn!("NPM is not installed. Installing NPM first...");
            npm_check().await;
        }

        let result = tokio::process::Command::new("sudo")
            .arg("npm")
            .arg("install")
            .arg("-g")
            .arg("yarn")
            .output()
            .await;

        let result = match result {
            Ok(o) => o,
            Err(e) => {
                fatal!("Failed to install Yarn: {}", e);
                return;
            }
        };

        if !result.status.success() {
            fatal!(
                "Failed to install Yarn: {}",
                String::from_utf8_lossy(&result.stderr)
            );
            return;
        }

        info!("Yarn installed successfully.");
    } else {
        info!("Yarn is installed.");
    }
}
