use crate::{registry::PluginRegistry, spawn_process, spawn_process_quiet, templates::Templates};
use anyhow::{Result, anyhow};
use common::{
    config::{Config, Framework},
    paths,
};
use std::path::{Path, PathBuf};
use tokio::fs;

pub struct ViteWorkspace {
    root: PathBuf,
}

impl ViteWorkspace {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        ViteWorkspace {
            root: path.as_ref().to_path_buf(),
        }
    }

    pub async fn init(&self, config: &Config) -> Result<()> {
        println!("Checking directories...");
        self.check_directories().await?;

        println!("Writing templates...");
        Templates::write_all(&self.root).await?;

        println!("Running npm install...");
        self.npm_install().await?;

        println!("Generating indices...");
        self.generate_html_indices(config).await?;
        self.generate_jsx_indices(config).await?;

        println!("Registering plugins...");

        let Some(plugin_directory) = paths::plugins() else {
            return Err(anyhow!(
                "Plugin directory not found. Please ensure plugins are set up correctly."
            ));
        };

        let mut registry = PluginRegistry::new(&plugin_directory);

        if let Err(e) = registry.init().await {
            return Err(anyhow!("Failed to initialize plugin registry: {}", e));
        }

        println!("Writing plugin registry...");
        let path = self.root.join("registry.js");
        registry.write(&path, Framework::React).await?;

        println!("Running prettier...");

        if let Err(_) = spawn_process_quiet("npm", &["run", "format"], Some(&self.root)).await {
            eprintln!("Failed to run prettier. Output will be ugly :(",);
        }

        println!("Running vite build...");
        self.vite_build().await?;

        println!("Project built!");

        Ok(())
    }

    async fn check_directories(&self) -> Result<()> {
        fs::create_dir_all(self.root.join("html")).await?;
        fs::create_dir_all(self.root.join("jsx")).await?;

        Ok(())
    }

    async fn npm_install(&self) -> Result<()> {
        let status = spawn_process_quiet("npm", &["install"], Some(&self.root)).await?;

        if !status.success() {
            return Err(anyhow!("npm install failed with status: {}", status));
        }

        Ok(())
    }

    async fn generate_html_indices(&self, config: &Config) -> Result<()> {
        for wc in &config.windows {
            let content = Templates::html_index(&wc.label);
            let path = self.root.join("html").join(format!("{}.html", wc.label));

            fs::write(path, content).await?;
        }

        Ok(())
    }

    async fn generate_jsx_indices(&self, config: &Config) -> Result<()> {
        let styles = paths::styles().ok_or_else(|| anyhow!("Failed to find styles directory"))?;
        let mut rd = fs::read_dir(&styles).await?;
        let mut styles = Vec::new();

        while let Some(e) = rd.next_entry().await.transpose() {
            let Ok(entry) = e else {
                continue;
            };

            if let Some(ext) = entry.path().extension() {
                if ext == "css" {
                    styles.push(entry.path().display().to_string());
                }
            }
        }

        for wc in &config.windows {
            let content = Templates::jsx_index(&wc.label, &styles);
            let path = self.root.join("jsx").join(format!("{}.jsx", wc.label));

            fs::write(path, content).await?;
        }
        Ok(())
    }

    async fn vite_build(&self) -> Result<()> {
        let status = spawn_process("npm", &["run", "build"], Some(&self.root)).await?;

        if !status.success() {
            return Err(anyhow!("vite build failed with status: {}", status));
        }

        Ok(())
    }
}
