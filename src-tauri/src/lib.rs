mod paths;
mod plugins;

use crate::plugins::{exec, get_plugins, read_plugin};
use gtk::{
    gdk::{traits::MonitorExt, Display, WindowTypeHint, RGBA},
    traits::{ContainerExt, GtkWindowExt, WidgetExt},
};
use gtk_layer_shell::{Edge, Layer, LayerShell};
use tauri::Manager;
use traccia::fatal;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![exec, get_plugins, read_plugin])
        .setup(|app| {
            let Some(tauri_window) = app.get_webview_window("main") else {
                fatal!("Failed to get the main webview window");
                std::process::exit(1);
            };

            if let Err(e) = tauri_window.hide() {
                fatal!("Failed to destroy the Tauri webview window: {}", e);
                std::process::exit(1);
            }

            let Ok(tauri_gtk_window) = tauri_window.gtk_window() else {
                fatal!("Failed to get the GTK window from the Tauri webview window");
                std::process::exit(1);
            };

            let Some(gtk_app) = tauri_gtk_window.application() else {
                fatal!("Failed to get the GTK application from the Tauri GTK window");
                std::process::exit(1);
            };

            let gtk_window = gtk::ApplicationWindow::new(&gtk_app);

            let Ok(vbox) = tauri_window.default_vbox() else {
                fatal!("Failed to get the VBox from the Tauri GTK window");
                std::process::exit(1);
            };

            tauri_gtk_window.remove(&vbox);
            gtk_window.add(&vbox);

            // Set up transparency
            gtk_window.set_app_paintable(true);

            // Set visual for transparency support
            if let Some(screen) = gtk::prelude::GtkWindowExt::screen(&gtk_window) {
                if let Some(visual) = screen.rgba_visual() {
                    gtk_window.set_visual(Some(&visual));
                }
            }

            gtk_window.init_layer_shell();
            gtk_window.set_layer(Layer::Top);

            if let Some(display) = Display::default() {
                let monitor = display
                    .primary_monitor()
                    .unwrap_or_else(|| display.monitor(0).expect("No monitors found."));

                let width = monitor.geometry().width();

                gtk_window.set_width_request(width - 100);
                // TODO: parse height from config file
                gtk_window.set_height_request(30);
                gtk_window.set_exclusive_zone(30);

                gtk_window.set_anchor(Edge::Top, true);
                gtk_window.set_anchor(Edge::Left, false);
                gtk_window.set_anchor(Edge::Right, false);
                gtk_window.set_anchor(Edge::Bottom, false);

                // Set window parameters
                gtk_window.set_keep_above(true);
                gtk_window.set_resizable(false);

                // Dock the window
                gtk_window.set_type_hint(WindowTypeHint::Dock);

                // Show the gtk window
                gtk_window.show_all();
            } else {
                fatal!("No displays were found. Aborting.");
                std::process::exit(1);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
