use crate::{Args, debug_mode, templates::Templates};
use common::{config::Config, io::Io};
use std::path::{Path, PathBuf};
use tokio::{fs, io};
use traccia::{debug, info};

pub struct ViteWorkspace {
    root: PathBuf,
    jsx: PathBuf,
    html: PathBuf,
}

impl ViteWorkspace {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let root = path.as_ref().to_path_buf();
        let jsx = root.join("jsx");
        let html = root.join("html");

        debug!("Creating frontend workspace at: {}", root.display());

        Self { root, jsx, html }
    }

    pub async fn init(&self, args: &Args, config: &Config) -> io::Result<()> {
        info!("Initializing vite workspace...");

        self.clean().await?;

        Templates::write_all(&self.root).await?;

        self.generate_indices(config).await?;

        if debug_mode() {
            debug!("Running npm install...");
        }

        Io::spawn_and_capture("npm", &["install"], Some(&self.root)).await?;

        if debug_mode() {
            debug!("Running prettier...");
        }

        Io::spawn_and_capture("npm", &["run", "format"], Some(&self.root)).await?;

        if debug_mode() {
            debug!("Building vite...");
        }

        if args.show_output {
            Io::spawn_with_output("npm", &["run", "build"], Some(&self.root)).await?;
        } else {
            Io::spawn_and_capture("npm", &["run", "build"], Some(&self.root)).await?;
        }

        info!("Workspace initialized successfully.");

        Ok(())
    }

    async fn clean(&self) -> io::Result<()> {
        for path in [&self.root, &self.jsx, &self.html] {
            if debug_mode() {
                debug!("Cleaning {}...", path.display());
            }

            Io::clean(path).await?;
        }

        if debug_mode() {
            debug!("Vite workspace cleaned.");
        }

        Ok(())
    }

    async fn generate_indices(&self, config: &Config) -> io::Result<()> {
        for w in &config.windows {
            let jsx_path = self.jsx.join(format!("{}.jsx", w.label));
            let html_path = self.html.join(format!("{}.html", w.label));

            // Transform styles and plugins paths to absolute paths
            let config_dir = Config::dir().expect("Cannot fail");
            let styles = w.styles.iter().map(|s| config_dir.join(s)).collect();
            let plugins = w.plugins.iter().map(|p| config_dir.join(p)).collect();

            if debug_mode() {
                debug!("Creating {}...", jsx_path.display());
            }

            fs::write(&jsx_path, Templates::jsx_index(&w.label, &styles, &plugins)).await?;

            if debug_mode() {
                debug!("Creating {}...", html_path.display());
            }

            fs::write(&html_path, Templates::html_index(&w.label)).await?;
        }

        Ok(())
    }
}
