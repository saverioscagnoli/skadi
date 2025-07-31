use crate::{spawn_process_output, spawn_process_quiet};
use anyhow::Result;
use nix::unistd::Uid;
use tokio::fs;
use which::which;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Distro {
    Arch,
    Debian,
    Fedora,
    Ubuntu,
    Unknown,
}

impl Distro {
    async fn detect_fallback() -> Self {
        let Ok(output) = spawn_process_output("lsb_release", &["-is"], None).await else {
            return Distro::Unknown;
        };

        let distro = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_lowercase();

        match distro.as_str() {
            "arch" => Distro::Arch,
            "debian" => Distro::Debian,
            "fedora" => Distro::Fedora,
            "ubuntu" => Distro::Ubuntu,
            _ => Distro::Unknown,
        }
    }

    pub async fn detect() -> Self {
        let Ok(os_release) = fs::read_to_string("/etc/os-release").await else {
            return Self::detect_fallback().await;
        };

        let Some(line) = os_release.lines().find(|l| l.starts_with("ID=")) else {
            return Self::detect_fallback().await;
        };

        let id = line
            .split('=')
            .nth(1)
            .unwrap_or("")
            .trim_matches('"')
            .to_lowercase();

        match id.as_str() {
            "arch" => Distro::Arch,
            "debian" => Distro::Debian,
            "fedora" => Distro::Fedora,
            "ubuntu" => Distro::Ubuntu,
            _ => Distro::Unknown,
        }
    }

    pub fn install_command<T: AsRef<str>>(&self, package: T) -> String {
        match self {
            Distro::Arch => format!("sudo pacman -S --noconfirm {}", package.as_ref()),
            Distro::Debian | Distro::Ubuntu => {
                format!("sudo apt-get install -y {}", package.as_ref())
            }
            Distro::Fedora => format!("sudo dnf install -y {}", package.as_ref()),
            Distro::Unknown => {
                format!("echo 'Unknown distro, cannot install {}'", package.as_ref())
            }
        }
    }
}

unsafe impl Send for Requirements {}
unsafe impl Sync for Requirements {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Requirements {
    pub distro: Distro,
}

impl Requirements {
    pub async fn new() -> Self {
        let distro = Distro::detect().await;
        Requirements { distro }
    }

    pub fn check<T: AsRef<str>>(&self, bin: T) -> bool {
        which(bin.as_ref()).is_ok()
    }

    pub fn check_root(&self) -> bool {
        Uid::effective().is_root()
    }

    pub async fn install_package<T: AsRef<str>>(&self, package: T) -> Result<()> {
        let command = self.distro.install_command(package);

        spawn_process_quiet("sh", &["-c", &command], None).await?;

        Ok(())
    }
}

unsafe impl Send for Distro {}
unsafe impl Sync for Distro {}
