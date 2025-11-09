use common::{config::Config, paths, templates::Templates, util};
use std::{error::Error, path::Path};
use traccia::{Style, debug, error};

/// The directory names inside the builds directory
/// this is specified to avoid cleaning unrelated files,
/// since the builds directory is inside ~/.local/share/wwwidgets
const DIRECTORY_NAMES: &[&str] = &["jsx", "html", "node_modules"];

fn clean<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn Error>> {
    if path.as_ref().exists() {
        std::fs::remove_dir_all(&path)?;
    }

    std::fs::create_dir_all(&path)?;

    Ok(())
}

fn write_templates<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn Error>> {
    let path = path.as_ref();

    if let Err(e) = std::fs::write(path.join("package.json"), Templates::PACKAGE_JSON) {
        return Err(format!("Could not write package.json to {}: {}", path.display(), e).into());
    }

    debug!("Wrote package.json to {}", path.display());

    if let Err(e) = std::fs::write(path.join("vite.config.js"), Templates::VITE_CONFIG) {
        return Err(format!(
            "Could not write vite.config.js to {}: {}",
            path.display(),
            e
        )
        .into());
    }

    debug!("Wrote vite.config.js to {}", path.display());

    Ok(())
}

pub fn generate_indices<P: AsRef<Path>>(config: &Config, root: P) -> Result<(), Box<dyn Error>> {
    let root = root.as_ref();
    let jsx_dir = root.join("jsx");
    let html_dir = root.join("html");

    for widget in &config.widgets {
        let path = if widget.index.is_absolute() {
            widget.index.clone()
        } else {
            paths::config_dir().join(&widget.index)
        };

        let path = match std::fs::canonicalize(path) {
            Ok(p) => p,
            Err(e) => {
                error!(
                    "Could not canonicalize index path for widget {}: {}",
                    widget.label, e
                );
                continue;
            }
        };

        if let Err(e) = std::fs::write(
            jsx_dir.join(format!("{}.jsx", widget.label)),
            Templates::jsx_index(path),
        ) {
            error!(
                "Could not write jsx index for widget {}: {}",
                widget.label, e
            );
            continue;
        }

        debug!(
            "Wrote jsx index for widget {} to {}",
            widget.label,
            jsx_dir.join(format!("{}.jsx", widget.label)).display()
        );

        if let Err(e) = std::fs::write(
            html_dir.join(format!("{}.html", widget.label)),
            Templates::html_index(&widget.label),
        ) {
            error!(
                "Could not write html index for widget {}: {}",
                widget.label, e
            );
            continue;
        }

        debug!(
            "Wrote html index for widget {} to {}",
            widget.label,
            html_dir.join(format!("{}.html", widget.label)).display()
        );
    }

    Ok(())
}

pub fn init(config: &Config) -> Result<(), Box<dyn Error>> {
    let local_dir = paths::local_dir();

    for dir in DIRECTORY_NAMES {
        let path = local_dir.join(dir);
        debug!("Cleaning {}", path.display());

        if let Err(e) = clean(&path) {
            error!("Could not clean directory {}: {}", path.display(), e);
        }
    }

    write_templates(&local_dir)?;
    generate_indices(config, &local_dir)?;

    // run yarn pipeline
    util::spawn_capture(
        format!(
            "cd {} && yarn && yarn format && yarn build",
            local_dir.display()
        ),
        |l| {
            println!("{}", l.dim());
        },
    )?;

    Ok(())
}
