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
    // Try to set some env variables to reduce GTK and WebKit logging noise
    std::env::set_var("G_MESSAGES_DEBUG", "");
    std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    std::env::set_var("WEBKIT_FORCE_SANDBOX", "0");

    // Redirect stderr to /dev/null for WebKit messages
    unsafe {
        let devnull = std::ffi::CString::new("/dev/null").unwrap();
        let fd = libc::open(devnull.as_ptr(), libc::O_WRONLY);
        if fd != -1 {
            libc::dup2(fd, 2); // Redirect stderr
            libc::close(fd);
        }
    }
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
