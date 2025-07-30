use anyhow::Result;
use common::config::Framework;
use std::path::{Path, PathBuf};
use tokio::fs;

use crate::templates::Templates;

pub struct PluginRegistry {
    root: PathBuf,
    paths: Vec<PathBuf>,
}

impl PluginRegistry {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        PluginRegistry {
            root: root.as_ref().to_path_buf(),
            paths: Vec::new(),
        }
    }

    pub async fn init(&mut self) -> Result<()> {
        let mut rd = fs::read_dir(&self.root).await?;

        while let Some(e) = rd.next_entry().await.transpose() {
            let Ok(entry) = e else {
                continue;
            };

            let path = entry.path();
            match path.extension().and_then(|s| s.to_str()) {
                Some("jsx") | Some("tsx") => self.paths.push(path),
                _ => continue,
            }
        }

        Ok(())
    }

    fn js_consts(&self) -> String {
        self.paths
            .iter()
            .enumerate()
            .map(|(i, path)| {
                format!(
                    "const Plugin{} = React.lazy(() => import(\"{}\"));",
                    i + 1,
                    path.display()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn js_return(&self) -> String {
        format!(
            "export default [{}];",
            (1..=self.paths.len())
                .map(|i| format!("Plugin{}", i))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    pub async fn write<P: AsRef<Path>>(&self, path: P, framework: Framework) -> Result<()> {
        let path = path.as_ref();

        let content = match framework {
            Framework::React => format!(
                r#"
                    {}
                    // This file is auto-generated.
                    // Do not edit manually.

                    import React from "react";

                    {}

                    {}
                    "#,
                Templates::mit_license(),
                self.js_consts(),
                self.js_return()
            ),

            _ => todo!("Only React framework is currently supported"),
        };

        fs::write(path, content).await?;

        Ok(())
    }
}
