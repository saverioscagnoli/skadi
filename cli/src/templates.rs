use err::{Result, SkadiError};
use phf::phf_map;
use std::path::Path;
use tokio::fs;

pub struct TemplateManager;

impl TemplateManager {
    const TEMPLATE_MAP: phf::Map<&'static str, &'static str> = phf_map! {
        "package.json" => include_str!("../../templates/package.json"),
        "vite.config.js" => include_str!("../../templates/vite.config.js"),
        "index.css" => include_str!("../../templates/index.css"),
    };

    pub async fn write<T: AsRef<str>, P: AsRef<Path>>(template: T, dir: P) -> Result<()> {
        let t = template.as_ref();
        let dir = dir.as_ref();

        let content = Self::TEMPLATE_MAP
            .get(t)
            .ok_or_else(|| SkadiError::ViteWorkspaceInit(format!("Template '{}' not found", t)))?;

        let path = dir.join(t);

        fs::write(&path, content)
            .await
            .map_err(|e| SkadiError::ViteWorkspaceInit(e.to_string()))?;

        Ok(())
    }
}
