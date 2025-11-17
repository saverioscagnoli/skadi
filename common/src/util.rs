use gtk4::{
    gdk::{
        self, Display,
        prelude::{DisplayExt, MonitorExt},
    },
    gio::prelude::ListModelExtManual,
};
use rand::{Rng, distr::Alphanumeric};
use std::{
    collections::HashMap,
    error::Error,
    io::{BufRead, BufReader, Write},
    process::{Command, ExitStatus, Stdio},
    sync::atomic::{AtomicBool, Ordering},
};
use traccia::{Colorize, debug};

static DEBUG: AtomicBool = AtomicBool::new(false);
static DEV: AtomicBool = AtomicBool::new(false);

pub fn debug() -> bool {
    DEBUG.load(Ordering::Relaxed)
}

pub fn set_debug(v: bool) {
    DEBUG.store(v, Ordering::Relaxed);
}

pub fn dev() -> bool {
    DEV.load(Ordering::Relaxed)
}

pub fn set_dev(v: bool) {
    DEV.store(v, Ordering::Relaxed);
}

pub fn ask_yes_no<F: Fn()>(logger: F) -> Result<bool, std::io::Error> {
    logger();
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(matches!(input.trim().to_lowercase().as_str(), "y" | "yes"))
}

pub fn disable_gtk_logs() {
    debug!("GTK logs disabled.");
    gtk4::glib::log_set_writer_func(|_, _| gtk4::glib::LogWriterOutput::Unhandled);
}

pub fn enumerate_monitors(display: &Display) -> HashMap<String, gdk::Monitor> {
    display
        .monitors()
        .iter::<gdk::Monitor>()
        .filter_map(Result::ok)
        .enumerate()
        .map(|(i, monitor)| {
            let Some(connector) = monitor.connector() else {
                return (format!("Monitor {}", i), monitor);
            };

            (connector.to_string(), monitor)
        })
        .collect()
}

pub fn spawn_capture<S: AsRef<str>, F: Fn(&str)>(
    command: S,
    logger: F,
) -> Result<ExitStatus, Box<dyn Error>> {
    let mut child = Command::new("bash")
        .arg("-c")
        .arg(command.as_ref())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    const MAX_LINES: usize = 5;

    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut line_buffer: Vec<String> = Vec::new();
        let mut lines_printed = 0;

        for line in reader.lines().map_while(Result::ok) {
            line_buffer.push(line);
            if line_buffer.len() > MAX_LINES {
                line_buffer.remove(0);
            }

            // Clear previous output
            if lines_printed > 0 {
                print!("\x1b[{}A\x1b[J", lines_printed);
            }

            for buffered_line in &line_buffer {
                logger(buffered_line);
            }

            lines_printed = line_buffer.len();
        }
    }

    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);

        for line in reader.lines().map_while(Result::ok) {
            eprintln!("{}", line.color(traccia::Color::Red));
        }
    }

    Ok(child.wait()?)
}

pub fn random_string(length: usize) -> String {
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(length)
        .map(char::from)
        .collect::<String>()
}
