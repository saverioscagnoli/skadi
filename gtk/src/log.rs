use gtk4::glib::LogWriterOutput;
use traccia::{Color, Colorize, LogLevel, Style};

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
