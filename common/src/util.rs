use gtk4::{
    gdk::{
        self, Display,
        prelude::{DisplayExt, MonitorExt},
    },
    gio::prelude::ListModelExtManual,
};
use std::collections::HashMap;
use traccia::debug;

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
