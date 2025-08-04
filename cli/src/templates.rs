use chrono::Datelike;
use phf::{Map, phf_map};
use std::path::{Path, PathBuf};
use tokio::{fs, io};
use traccia::debug;

pub struct Templates;

impl Templates {
    const FILES: Map<&'static str, &'static str> = phf_map! {
        "index.css" => include_str!("../../templates/index.css"),
        "package.json" => include_str!("../../templates/package.json"),
        "tailwind.config.js" => include_str!("../../templates/tailwind.config.js"),
        "utils.js" => include_str!("../../templates/utils.js"),
        "vite.config.js" => include_str!("../../templates/vite.config.js")
    };

    pub async fn write_all<P: AsRef<Path>>(dir: P) -> io::Result<()> {
        let dir = dir.as_ref();

        for (name, content) in Self::FILES.entries() {
            let path = dir.join(name);

            fs::write(&path, content).await?;

            debug!("Wrote template file: {}", path.display());
        }

        Ok(())
    }

    pub fn html_index<T: AsRef<str>>(label: T) -> String {
        let label = label.as_ref();

        format!(
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
        )
    }

    pub fn jsx_index<L: AsRef<str>>(
        label: L,
        styles: &Vec<PathBuf>,
        plugins: &Vec<PathBuf>,
    ) -> String {
        let label = label.as_ref();

        format!(
            r#"
            {}

            import {{ createRoot }} from "react-dom/client";
            import React from "react";
            import {{ exec, useListen }} from "../utils.js";

            import "../index.css";

            // Styles
            {}

            // Plugins
            const plugins = [
            {}
            ];

            const LABEL = "{}"; 

            createRoot(document.getElementById("root")).render(
              <div className="w-screen h-screen {}-window">
                {{plugins.map(Plugin => (
                  <Plugin key={{Plugin.name}} exec={{exec}} useListen={{useListen}} />
                ))}}
              </div>
            );
            "#,
            Self::mit_license(),
            styles
                .iter()
                .map(|p| format!("import \"{}\";", p.display()))
                .collect::<Vec<_>>()
                .join(",\n"),
            plugins
                .iter()
                .map(|p| format!("React.lazy(() => import(\"{}\"))", p.display()))
                .collect::<Vec<_>>()
                .join(",\n"),
            label,
            label
        )
    }

    pub fn mit_license() -> String {
        let year = chrono::Utc::now().year();

        format!(
            r#"
        // MIT License
        //
        // Copyright (c) {} Saverio Scagnoli
        // 
        // Permission is hereby granted, free of charge, to any person obtaining a copy
        // of this software and associated documentation files (the "Software"), to deal
        // in the Software without restriction, including without limitation the rights
        // to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
        // copies of the Software, and to permit persons to whom the Software is
        // furnished to do so, subject to the following conditions:
        // 
        // The above copyright notice and this permission notice shall be included in all
        // copies or substantial portions of the Software.
        // 
        // THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
        // IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
        // FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
        // AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
        // LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
        // OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
        // SOFTWARE.
        "#,
            year
        )
    }
}
