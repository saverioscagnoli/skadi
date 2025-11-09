use gtk4::{
    gdk::{
        self, Display,
        prelude::{DisplayExt, MonitorExt},
    },
    gio::prelude::ListModelExtManual,
};
use std::{
    collections::HashMap,
    error::Error,
    io::{BufRead, BufReader, Write},
    process::{Command, ExitStatus, Stdio},
};
use traccia::{Colorize, debug};

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

    // Handle stdout
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);

        for line in reader.lines().map_while(Result::ok) {
            logger(&line);
        }
    }

    // Handle stderr
    if let Some(stderr) = child.stderr.take() {
        let reader = BufReader::new(stderr);

        for line in reader.lines().map_while(Result::ok) {
            eprintln!("{}", line.color(traccia::Color::Red));
        }
    }

    Ok(child.wait()?)
}
