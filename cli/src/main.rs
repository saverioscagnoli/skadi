use std::process::Command;
use std::io::{self, Write};
use std::fs;

fn detect_distro() -> Option<String> {
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("ID=") {
                return Some(line[3..].trim_matches('"').to_string());
            }
        }
    }
    None
}

fn install_node_for_distro(distro: &str) -> bool {
    let install_cmd = match distro {
        "ubuntu" | "debian" => "curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash - && sudo apt-get install -y nodejs",
        "fedora" => "curl -fsSL https://rpm.nodesource.com/setup_lts.x | sudo bash - && sudo dnf install -y nodejs",
        "centos" | "rhel" => "curl -fsSL https://rpm.nodesource.com/setup_lts.x | sudo bash - && sudo yum install -y nodejs",
        "arch" => "sudo pacman -Sy --noconfirm nodejs npm",
        _ => {
            println!("Unsupported or unknown distro: {}. Please install Node.js manually.", distro);
            return false;
        }
    };

    let status = Command::new("sh")
        .arg("-c")
        .arg(install_cmd)
        .status()
        .expect("Failed to run Node.js install command");
    status.success()
}

fn main() {
    // Check Node.js
    let node_installed = match Command::new("node").arg("--version").output() {
        Ok(output) if output.status.success() => {
            println!("Node.js is installed: {}", String::from_utf8_lossy(&output.stdout).trim());
            true
        }
        _ => {
            println!("Node.js is not installed.");
            false
        }
    };

    // Install Node.js if not installed
    if !node_installed {
        print!("Install Node.js? [y/N]: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if input.trim().eq_ignore_ascii_case("y") {
            if let Some(distro) = detect_distro() {
                if install_node_for_distro(&distro) {
                    println!("Node.js installed successfully.");
                } else {
                    println!("Failed to install Node.js.");
                }
            } else {
                println!("Could not detect Linux distribution. Please install Node.js manually.");
            }
        }
    }

    let yarn_installed = match Command::new("yarn").arg("--version").output() {
        Ok(output) if output.status.success() => {
            println!("Yarn is installed: {}", String::from_utf8_lossy(&output.stdout).trim());
            true
        }
        _ => {
            println!("Yarn is not installed.");
            false
        }
    };

    // Install Yarn with npm if not installed
    if !yarn_installed {
        print!("Install Yarn? [y/N]: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        if input.trim().eq_ignore_ascii_case("y") {
            let status = Command::new("sudo")
                .args(&["npm", "install", "-g", "yarn"])
                .status()
                .expect("Failed to run Yarn install command");
            if status.success() {
                println!("Yarn installed successfully.");
            } else {
                println!("Failed to install Yarn.");
            }
        }
    }
}