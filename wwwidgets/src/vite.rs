use common::{config::Config, paths, templates::Templates, util};
use std::{error::Error, path::Path, thread, time::Duration};
use traccia::{Style, debug, error, info};

/// The directory names inside the builds directory
/// this is specified to avoid cleaning unrelated files,
/// since the builds directory is inside ~/.local/share/wwwidgets
const DIRECTORY_NAMES: &[&str] = &["jsx", "html"];

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

    if let Err(e) = std::fs::write(path.join("utils.js"), Templates::UTILS_JS) {
        return Err(format!("Could not write utils.js to {}: {}", path.display(), e).into());
    }

    debug!("Wrote utils.js to {}", path.display());

    // Write types and hooks to .config dir
    let config_dir = paths::config_dir();

    if let Err(e) = std::fs::write(
        config_dir.join("use-backend.ts"),
        Templates::USE_BACKEND_HOOK,
    ) {
        return Err(format!(
            "Could not write use-backend.ts to {}: {}",
            config_dir.display(),
            e
        )
        .into());
    }

    debug!("Wrote use-backend.ts to {}", config_dir.display());

    if let Err(e) = std::fs::write(config_dir.join("types.d.ts"), Templates::TYPES_D_TS) {
        return Err(format!(
            "Could not write types.d.ts to {}: {}",
            config_dir.display(),
            e
        )
        .into());
    }

    debug!("Wrote types.d.ts to {}", config_dir.display());

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

        if let Err(e) = std::fs::write(
            root.join("index.css"),
            Templates::css_index(&paths::config_dir()),
        ) {
            error!(
                "Could not write index.css to {}: {}",
                root.join("index.css").display(),
                e
            );
        }

        debug!("Wrote index.css to {}", root.join("index.css").display());
    }

    Ok(())
}

fn wait_for_vite_server() -> Result<(), Box<dyn Error>> {
    debug!("Waiting for Vite dev server to start...");

    let max_attempts = 100;
    let mut attempts = 0;

    while attempts < max_attempts {
        // Check if Vite server is responding
        let is_ready = match ureq::get("http://localhost:5173/").call() {
            Ok(_) => true,
            Err(e) => {
                match e {
                    ureq::Error::StatusCode(_) => true, // Server responded with error status (like 404) (don't care about that)
                    _ => false,                         // Connection error - server not ready
                }
            }
        };

        if is_ready {
            info!("Vite dev server is up and running.");
            return Ok(());
        }

        thread::sleep(Duration::from_millis(300));
        attempts += 1;
    }

    Err("Vite dev server failed to start within 30 seconds".into())
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
        format!("cd {} && yarn && yarn format", local_dir.display()),
        |l| {
            println!("{}", l.dim());
        },
    )?;

    if util::dev() {
        debug!("Development mode enabled, serving vite server instead of building");
        thread::spawn(move || {
            if let Err(e) =
                util::spawn_capture(format!("cd {} && yarn dev", local_dir.display()), |l| {
                    println!("{}", l.dim());
                })
            {
                error!("Failed to start vite dev server: {}", e);
            }
        });

        wait_for_vite_server()?;
    } else {
        util::spawn_capture(format!("cd {} && yarn build", local_dir.display()), |l| {
            println!("{}", l.dim());
        })?;
    }
    Ok(())
}
