use common::io::{Io, SpawnResult};
use tokio::{fs, io};
use traccia::{debug, info, warn};
use which::which;

/// Simple enum to represent different Linux distributions
/// This is used to determine the package manager and installation commands
/// during the requirements check
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Distro {
    Arch,
    Debian,
    Fedora,
    Ubuntu,
    Unknown,
}

impl Distro {
    /// If /etc/os-release is not available, we fallback to using lsb_release
    /// This is a more reliable way to detect the distribution
    /// but it requires lsb_release to be installed
    async fn detect_fallback() -> Self {
        let Ok(r) = Io::spawn_and_capture("lsb_release", &["-is"]).await else {
            return Distro::Unknown;
        };

        let Some(outptut) = r.stdout else {
            return Distro::Unknown;
        };

        let distro = outptut.trim().to_lowercase();

        match distro.as_str() {
            "arch" => Distro::Arch,
            "debian" => Distro::Debian,
            "fedora" => Distro::Fedora,
            "ubuntu" => Distro::Ubuntu,
            _ => Distro::Unknown,
        }
    }

    /// Detect the Linux distribution by reading /etc/os-release
    /// This is the preferred method as it is more reliable and does not require
    /// additional tools like lsb_release
    ///
    /// Fallbacks to using lsb_release if /etc/os-release is not available
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

    /// Returns the string that would be used to install a package
    /// on the current distribution
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

/// Requirements struct to hold the detected distribution
/// and provide methods to check for required binaries and install packages
/// without checking every time
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Requirements {
    pub distro: Distro,
}

impl Requirements {
    const REQUIRED_BINARIES: [&'static str; 2] = ["node", "npm"];

    pub async fn new() -> Self {
        let distro = Distro::detect().await;
        Requirements { distro }
    }

    pub fn check<T: AsRef<str>>(&self, bin: T) -> bool {
        which(bin.as_ref()).is_ok()
    }

    pub async fn install_package<T: AsRef<str>>(&self, package: T) -> io::Result<SpawnResult> {
        let command = self.distro.install_command(package);

        Io::spawn_and_capture("sh", &["-c", &command]).await
    }

    pub async fn check_all(&self) -> io::Result<()> {
        let mut to_install = Vec::new();

        debug!("Detected distribution: {:?}", self.distro);
        info!(
            "Checking requirements: {}",
            Requirements::REQUIRED_BINARIES.join(", ")
        );

        for bin in Requirements::REQUIRED_BINARIES.iter() {
            if !self.check(bin) {
                to_install.push(bin.to_string());
            }
        }

        if to_install.is_empty() {
            Io::clear_line().await;
            info!("All requirements are met.");
            return Ok(());
        }

        Io::clear_line().await;
        warn!(
            "Missing required binaries: {}. Installing...",
            to_install.join(", ")
        );

        let packages = to_install.join(" ");
        let output = self.install_package(&packages).await?;

        if let Some(stderr) = output.stderr {
            if !stderr.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to install packages: {}", stderr),
                ));
            }
        }

        Io::clear_line().await;
        info!("Successfully installed required packages: {}", packages);

        Ok(())
    }
}
