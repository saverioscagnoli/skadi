use common::{config::Config, paths};
use err::{Result, SkadiError};
use futures::future;
use std::{
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::fs;
use traccia::info;

use crate::{
    plugins::{Framework, PluginRegistry},
    templates::TemplateManager,
};

pub struct ViteWorkspace {
    plugin_registry: PluginRegistry,
    dir: PathBuf,
}

impl ViteWorkspace {
    pub fn init<P: AsRef<Path>, Q: AsRef<Path>>(dir: P, plugin_dir: Q) -> Self {
        let dir = dir.as_ref().to_path_buf();
        let plugin_dir = plugin_dir.as_ref().to_path_buf();

        info!("Initializng vite workspace at {}", dir.display());

        Self {
            plugin_registry: PluginRegistry::new(&plugin_dir),
            dir,
        }
    }

    pub async fn generate(&mut self, config: &Config) -> Result<()> {
        info!("Writing necessary templates...");

        TemplateManager::write("package.json", &self.dir).await?;
        TemplateManager::write("vite.config.js", &self.dir).await?;
        TemplateManager::write("index.css", &self.dir).await?;

        info!("Running a `yarn install`...");

        self.yarn_install().await?;

        info!("Generating HTML and JSX indices...");

        let mut handles = Vec::new();

        for wc in &config.windows {
            let label_html = wc.label.clone();
            let label_jsx = wc.label.clone();

            let html_handle = tokio::spawn(async move {
                let html = PluginRegistry::gen_html_index(&label_html);
                let file_path = paths::html_indices()
                    .ok_or(SkadiError::ViteWorkspaceInit(
                        "Failed to find or create HTML indices directory".to_string(),
                    ))
                    .unwrap()
                    .join(format!("{}.html", label_html));

                fs::write(&file_path, html)
                    .await
                    .map_err(|e| SkadiError::ViteWorkspaceInit(e.to_string()))
                    .unwrap();
            });

            let jsx_handle = tokio::spawn(async move {
                let jsx = PluginRegistry::gen_jsx_index(&label_jsx);

                let file_path = paths::jsx_indices()
                    .ok_or(SkadiError::ViteWorkspaceInit(
                        "Failed to find or create JSX indices directory".to_string(),
                    ))
                    .unwrap()
                    .join(format!("{}.jsx", label_jsx));

                fs::write(&file_path, jsx)
                    .await
                    .map_err(|e| SkadiError::ViteWorkspaceInit(e.to_string()))
                    .unwrap();
            });

            handles.push(html_handle);
            handles.push(jsx_handle);
        }

        future::join_all(handles).await;

        info!("Generating plugin registry...");

        self.plugin_registry.register_all().await?;
        self.plugin_registry
            .write(&self.dir.join("registry.js"), Framework::React)
            .await?;

        info!("Running a `yarn build`...");

        self.yarn_build().await?;

        info!("Project built successfully!");

        Ok(())
    }

    async fn yarn_build(&self) -> Result<()> {
        let status = tokio::process::Command::new("yarn")
            .arg("build")
            .current_dir(&self.dir)
            .status()
            .await
            .map_err(|e| SkadiError::ViteWorkspaceInit(e.to_string()))?;

        if !status.success() {
            return Err(SkadiError::ViteWorkspaceInit(
                "Failed to run `yarn build`".to_string(),
            ));
        }

        Ok(())
    }

    async fn yarn_install(&self) -> Result<()> {
        let status = tokio::process::Command::new("yarn")
            .arg("install")
            .current_dir(&self.dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map_err(|e| SkadiError::ViteWorkspaceInit(e.to_string()))?;

        if !status.success() {
            return Err(SkadiError::ViteWorkspaceInit(
                "Failed to run `yarn install`".to_string(),
            ));
        }

        Ok(())
    }
}
