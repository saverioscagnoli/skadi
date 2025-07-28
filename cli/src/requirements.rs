use err::{Result, SkadiError};
use std::sync::LazyLock;
use traccia::{info, warn};

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

    fn install_command(&self, package: &str) -> Option<String> {
        match self {
            Distro::Ubuntu => format!("sudo apt-get install -y {}", package).into(),
            Distro::Fedora => format!("sudo dnf install -y {}", package).into(),
            Distro::Arch => format!("sudo pacman -S --noconfirm {}", package).into(),
            Distro::Unknown => None,
        }
    }
}

static DISTRO: LazyLock<Distro> = LazyLock::new(Distro::detect);

pub struct Requirements;

impl Requirements {
    pub async fn check() -> Result<()> {
        info!("Checking requirements...");

        Self::check_node().await?;
        Self::check_npm().await?;
        Self::check_yarn().await?;

        Ok(())
    }

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

    pub async fn install_node() -> Result<()> {
        let Some(command) = DISTRO.install_command("nodejs") else {
            return Err(SkadiError::RequirementsCheck(
                "Unsupported Linux distribution".into(),
            ));
        };

        let result = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .await;

        let output = match result {
            Ok(o) => o,
            Err(e) => return Err(SkadiError::RequirementsCheck(e.to_string())),
        };

        if !output.status.success() {
            return Err(SkadiError::RequirementsCheck(
                String::from_utf8_lossy(&output.stderr).into(),
            ));
        }

        Ok(())
    }

    pub async fn install_npm() -> Result<()> {
        let Some(command) = DISTRO.install_command("npm") else {
            return Err(SkadiError::RequirementsCheck(
                "Unsupported Linux distribution".into(),
            ));
        };

        let result = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .await;

        let output = match result {
            Ok(o) => o,
            Err(e) => return Err(SkadiError::RequirementsCheck(e.to_string())),
        };

        if !output.status.success() {
            return Err(SkadiError::RequirementsCheck(
                String::from_utf8_lossy(&output.stderr).into(),
            ));
        }

        Ok(())
    }

    pub async fn install_yarn() -> Result<()> {
        let result = tokio::process::Command::new("sudo")
            .arg("npm")
            .arg("install")
            .arg("-g")
            .arg("yarn")
            .output()
            .await;

        let output = match result {
            Ok(o) => o,
            Err(e) => return Err(SkadiError::RequirementsCheck(e.to_string())),
        };

        if !output.status.success() {
            return Err(SkadiError::RequirementsCheck(
                String::from_utf8_lossy(&output.stderr).into(),
            ));
        }

        Ok(())
    }

    async fn check_node() -> Result<()> {
        info!("Checking if Node.js is installed...");

        if Self::node_installed().await {
            info!("Node.js is installed.");
            return Ok(());
        }

        warn!("Node.js is not installed. Attempting to install...");

        Self::install_node().await?;

        info!("Node.js installed successfully.");

        Ok(())
    }

    async fn check_npm() -> Result<()> {
        info!("Checking if npm is installed...");

        if Self::npm_installed().await {
            info!("npm is installed.");
            return Ok(());
        }

        warn!("npm is not installed. Attempting to install...");

        Self::install_npm().await?;

        info!("npm installed successfully.");

        Ok(())
    }

    async fn check_yarn() -> Result<()> {
        info!("Checking if Yarn is installed...");

        if Self::yarn_installed().await {
            info!("Yarn is installed.");
            return Ok(());
        }

        warn!("Yarn is not installed. Attempting to install...");

        Self::install_yarn().await?;

        info!("Yarn installed successfully.");

        Ok(())
    }
}
