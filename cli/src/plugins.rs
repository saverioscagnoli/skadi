use err::{Result, SkadiError};
use std::path::{Path, PathBuf};
use textwrap::dedent;
use tokio::fs;
use traccia::{info, warn};

pub enum Framework {
    React,
    Vue,
    Svelte,
    Solid,
    Vanilla,
}

pub struct PluginRegistry {
    dir: PathBuf,
    plugin_paths: Vec<PathBuf>,
}

impl PluginRegistry {
    pub fn new<P: AsRef<Path>>(dir: P) -> Self {
        Self {
            dir: dir.as_ref().to_path_buf(),
            plugin_paths: Vec::new(),
        }
    }

    pub(super) async fn register_all(&mut self) -> Result<()> {
        let mut rd = fs::read_dir(&self.dir)
            .await
            .map_err(|e| SkadiError::ViteWorkspaceInit(e.to_string()))?;

        let mut paths = Vec::new();

        while let Some(e) = rd.next_entry().await.transpose() {
            let Ok(entry) = e else {
                warn!("Failed to read plugin entry, skipping");
                continue;
            };

            let path = entry.path();

            match path.extension().and_then(|s| s.to_str()) {
                Some("jsx") | Some("tsx") => {
                    info!("Registering plugin: {}", path.display());
                    paths.push(path)
                }
                _ => continue,
            }
        }

        self.plugin_paths = paths;

        Ok(())
    }

    pub fn gen_html_index<L: AsRef<str>>(label: L) -> String {
        let label = label.as_ref();

        dedent(&format!(
            r#"
            <!DOCTYPE html>
            <html lang="en">
                <head>
                    <meta charset="UTF-8" />
                    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                    <title>{}</title>
                </head>
                <body>
                    <div id="root"></div>
                    <script type="module" src="/jsx/{}.jsx"></script>
                </body>
            </html>
        "#,
            label, label
        ))
    }

    pub fn gen_jsx_index<L: AsRef<str>>(label: L) -> String {
        let label = label.as_ref();

        dedent(&format!(
            r#"
            import {{ createRoot }} from "react-dom/client";
            import plugins from "../registry.js";
            // import {{ exec, useListen }} from "../util.js";

            import "../index.css";

            const LABEL = "{}"; 

            createRoot(document.getElementById("root")).render(
              <div className="w-screen h-screen {}-window">
                {{plugins.map(Plugin => (
                  <Plugin key={{Plugin.name}}  />
                ))}}
              </div>
            );
            "#,
            label, label
        ))
    }

    fn react_exports(&self) -> String {
        self.plugin_paths
            .iter()
            .enumerate()
            .map(|(i, path)| {
                format!(
                    "const Plugin{} = () => React.lazy(() => import(\"{}\"));",
                    i + 1,
                    path.display()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn js_return_statement(&self) -> String {
        format!(
            "export default [{}];",
            (1..=self.plugin_paths.len())
                .map(|i| format!("Plugin{}", i))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    pub async fn write<P: AsRef<Path>>(&self, path: P, framework: Framework) -> Result<()> {
        let path = path.as_ref();

        let content = match framework {
            Framework::React => {
                format!(
                    r#"
                    // This file is auto-generated.
                    // Do not edit manually.
                    import React from "react";

                    {}

                    {}
                    "#,
                    self.react_exports(),
                    self.js_return_statement()
                )
            }
            _ => todo!("Only React framework is currently supported"),
        };

        fs::write(path, content)
            .await
            .map_err(|e| SkadiError::ViteWorkspaceInit(e.to_string()))?;

        Ok(())
    }
}
