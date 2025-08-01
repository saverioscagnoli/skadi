use crate::{
    spawn_process_output, spawn_process_quiet, spinner::SpinnerHandle, templates::Templates,
};
use anyhow::{Result, anyhow};
use common::{config::Config, paths};
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

    pub async fn init(&self, config: &Config, mut spinner: SpinnerHandle) -> Result<()> {
        spinner.update_message("Checking directories...");
        self.check_directories().await?;

        spinner.update_message("Writing templates...");
        Templates::write_all(&self.root).await?;

        spinner.update_message("Running npm install...");
        self.npm_install().await?;

        spinner.update_message("Generating indices...");
        self.generate_html_indices(config).await?;
        self.generate_jsx_indices(config).await?;

        spinner.update_message("Running prettier...");

        if let Err(_) = spawn_process_quiet("npm", &["run", "format"], Some(&self.root)).await {
            eprintln!("Failed to run prettier. Output will be ugly :(",);
        }

        spinner.update_message("Running vite build...");
        self.vite_build().await?;

        spinner.finish_with_symbol_and_message("🪄", "Frontend built!");

        Ok(())
    }

    async fn check_directories(&self) -> Result<()> {
        let html = self.root.join("html");
        let jsx = self.root.join("jsx");

        fs::create_dir_all(&html).await?;
        fs::create_dir_all(&jsx).await?;

        self.clean_directory(&html).await?;
        self.clean_directory(&jsx).await?;

        Ok(())
    }

    async fn clean_directory<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
        let mut entries = fs::read_dir(dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path: PathBuf = entry.path();

            if path.is_dir() {
                fs::remove_dir_all(&path).await?;
            } else {
                fs::remove_file(&path).await?;
            }
        }

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
            let paths = wc
                .plugins
                .iter()
                .map(|p| {
                    if p.is_absolute() {
                        p.clone()
                    } else {
                        paths::config()
                            .map(|base| base.join(p))
                            .unwrap_or_else(|| p.clone())
                    }
                })
                .collect::<Vec<_>>();

            let content = Templates::jsx_index(&wc.label, &styles, &paths);
            let path = self.root.join("jsx").join(format!("{}.jsx", wc.label));

            fs::write(path, content).await?;
        }
        Ok(())
    }

    async fn vite_build(&self) -> Result<()> {
        let output = spawn_process_output("npm", &["run", "build"], Some(&self.root)).await?;

        if !output.status.success() {
            return Err(anyhow!(
                "vite build failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }
}
