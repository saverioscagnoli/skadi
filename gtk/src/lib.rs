mod traits;

use crate::traits::GtkSetup;
use common::config::Config;
use gtk4::{
    gdk,
    gio::prelude::{ApplicationExt, ApplicationExtManual},
    glib::LogWriterOutput,
    prelude::{GtkWindowExt, WidgetExt},
};
use traccia::{Color, Colorize, LogLevel, Style, fatal, info};

struct CustomFormatter;

impl traccia::Formatter for CustomFormatter {
    fn format(&self, record: &traccia::Record) -> String {
        let timestamp = chrono::Local::now()
            .format("%b %d %H:%M:%S")
            .to_string()
            .color(Color::Cyan)
            .dim();

        format!(
            "{} [{}] {}: {}",
            timestamp,
            record.target.dim(),
            record.level.default_coloring().to_lowercase(),
            record.message
        )
    }
}

fn log_level() -> LogLevel {
    if cfg!(debug_assertions) {
        LogLevel::Debug
    } else {
        LogLevel::Info
    }
}

fn disable_gtk_logs() {
    gtk4::glib::log_set_writer_func(|_log_domain, _log_level| LogWriterOutput::Unhandled);
}

pub fn setup_logging() {
    disable_gtk_logs();

    // Setup logger
    traccia::init_with_config(traccia::Config {
        level: log_level(),
        format: Some(Box::new(CustomFormatter)),
        ..Default::default()
    });
}

pub fn run(config: Config) {
    setup_logging();

    if let Err(e) = gtk4::init() {
        fatal!("Failed to initialize GTK: {}", e);
        return;
    }

    let app = config.create_app();

    app.connect_startup(|app| {
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

        app.activate();
    });

    app.connect_activate(move |app| {
        let windows = match config.setup_windows(&app) {
            Ok(w) => w,
            Err(e) => {
                fatal!("Failed to setup windows: {}", e);
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
