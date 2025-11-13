use std::path::{Path, PathBuf};

use crate::paths;

pub struct Templates;

impl Templates {
    pub const DEFAULT_CONFIG: &'static str = include_str!("../../templates/config.default.json");
    pub const LICENSE: &'static str = include_str!("../../templates/LICENSE");
    pub const PACKAGE_JSON: &'static str = include_str!("../../templates/package.json");
    pub const VITE_CONFIG: &'static str = include_str!("../../templates/vite.config.js");
    pub const BACKEND_TS: &'static str = include_str!("../../templates/backend.ts");
    pub const USE_BACKEND_HOOK: &'static str = include_str!("../../templates/use-backend.ts");

    pub fn commented_license() -> String {
        Self::LICENSE
            .lines()
            .map(|line| format!("// {}", line))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn jsx_index(index: PathBuf) -> String {
        format!(
            r#"
            // This file was automatically generated, do not edit directly.
            {}

            import {{ createRoot }} from "react-dom/client";
            import * as backend from "../backend.ts";
            import {{ BackendContext }} from "{}";

            import Index from "{}";
            import "../index.css";

            createRoot(document.getElementById("root")).render(
                <BackendContext.Provider value={{backend}}>
                    <Index />
                </BackendContext.Provider>
            );
            "#,
            Self::commented_license(),
            paths::config_dir().join("use-backend.ts").display(),
            index.display()
        )
    }

    /// Sets the widget label as the title of the document,
    /// so that it can be used for backend communication, to identify the widget.
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

    pub fn css_index<P: AsRef<Path>>(config_dir: P) -> String {
        format!(
            r#"
            @import "tailwindcss";

            @source "{}/**/*.{{html,css,js,ts,jsx,tsx}}";
            "#,
            config_dir.as_ref().display()
        )
    }
}
