use std::path::PathBuf;

pub struct Templates;

impl Templates {
    pub const DEFAULT_CONFIG: &'static str = include_str!("../../templates/config.default.json");
    pub const LICENSE: &'static str = include_str!("../../templates/LICENSE");
    pub const PACKAGE_JSON: &'static str = include_str!("../../templates/package.json");
    pub const VITE_CONFIG: &'static str = include_str!("../../templates/vite.config.js");

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

            import Index from "{}";

            createRoot(document.getElementById("root")).render(<div><Index /></div>);
            "#,
            Self::commented_license(),
            index.display()
        )
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
}
