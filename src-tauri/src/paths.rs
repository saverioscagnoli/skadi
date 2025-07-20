use std::{fs, path::PathBuf};

pub struct Paths;

impl Paths {
    pub fn config() -> Option<PathBuf> {
        dirs::config_dir().map(|mut path| {
            path.push("skadi");
            path
        })
    }

    pub fn plugins() -> Option<PathBuf> {
        let mut conf = Self::config()?;
        conf.push("plugins");

        if !conf.exists() {
            fs::create_dir_all(&conf).ok()?;
        }

        Some(conf)
    }
}
