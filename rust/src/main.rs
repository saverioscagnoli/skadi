mod api;
mod config;
mod error;
mod log;

use crate::config::Config;
use gtk4::{
    gdk,
    gio::prelude::{ApplicationExt, ApplicationExtManual},
    prelude::{GtkWindowExt, WidgetExt},
};
use tokio::sync::oneshot;
use traccia::{fatal, info};

#[tokio::main]
async fn main() {
    log::setup_logging();

    let config = match Config::parse() {
        Ok(config) => config,
        Err(e) => {
            fatal!("Failed to parse configuration: {}", e);
            return;
        }
    };

    let (ready_tx, ready_rx) = oneshot::channel::<()>();

    let Some(cd) = std::env::current_dir().ok() else {
        fatal!("Failed to get current directory");
        return;
    };

    let Some(parent) = cd.parent() else {
        fatal!("Failed to get parent directory of current directory");
        return;
    };

    let dist = parent.join("dist");

    tokio::spawn(async move {
        if let Err(e) = api::asset_server(config.port, dist, ready_tx).await {
            fatal!("Failed to run server: {}", e);
            return;
        }
    });

    // Wait for the server to signal readiness
    if let Err(e) = ready_rx.await {
        fatal!("Failed to receive ready signal from server: {}", e);
        return;
    }

    if let Err(e) = gtk4::init() {
        fatal!("Failed to initialize GTK: {}", e);
        return;
    }

    let app = config.create_app();

    app.connect_startup(|_| {
        let provider = gtk4::CssProvider::new();
        let css_str = r"
            window {
                background-color: transparent;
            }            
        ";

        provider.load_from_string(css_str);

        gtk4::style_context_add_provider_for_display(
            &gdk::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        if let Some(settings) = gtk4::Settings::default() {
            settings.set_gtk_theme_name(Some(""));
            settings.set_gtk_icon_theme_name(Some(""));
        }
    });

    app.connect_activate(move |app| {
        let windows = match config.setup_windows(app) {
            Ok(w) => w,
            Err(e) => {
                fatal!("Failed to setup window: {}", e);
                return;
            }
        };

        for w in windows {
            info!(
                "Created window '{}' {}x{}",
                w.title().unwrap_or("Untitled".into()),
                w.width_request(),
                w.height_request()
            );

            w.present();
        }
    });

    app.run();
}
