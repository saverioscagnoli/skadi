use gtk4::{
    gdk::{
        self, Display,
        prelude::{DisplayExt, MonitorExt},
    },
    gio::prelude::ListModelExtManual,
};
use std::{collections::HashMap, io::Write};
use traccia::debug;

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
