mod config;
mod error;
mod log;
mod server;

use std::{
    fs,
    net::{TcpListener, TcpStream},
    path,
};

use gtk4::{
    gdk,
    gio::prelude::{ApplicationExt, ApplicationExtManual},
    prelude::{GtkWindowExt, WidgetExt},
    Application,
};
use tokio::sync::oneshot;
use traccia::{fatal, info};

use crate::{config::Config, server::LocalServer};

fn load_css() {
    let provider = gtk4::CssProvider::new();
    let css_str = r"
        * {
            all: unset;
        }

        window {
            background-color: transparent;
        }

        webview {
            background-color: transparent;
            border: 0;

    app.connect_s
            outline: 0;
            margin: 0;
            padding: 0;
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
}

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

    let server = match LocalServer::new(config.port).await {
        Ok(server) => {
            let Some(cd) = std::env::current_dir().ok() else {
                fatal!("Failed to get current directory");
                return;
            };

            let Some(parent) = cd.parent() else {
                fatal!("Failed to get parent directory of current directory");
                return;
            };

            let dist = parent.join("dist");

            server.with_root(dist)
        }
        Err(e) => {
            fatal!("Failed to start server: {}", e);
            return;
        }
    };

    let (ready_tx, ready_rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        if let Err(e) = server.run(ready_tx).await {
            fatal!("Server error: {}", e);
        }
    });

    if let Err(_) = ready_rx.await {
        fatal!("Failed to receive server ready signal");
        return;
    }

    if let Err(e) = gtk4::init() {
        fatal!("Failed to initialize GTK: {}", e);
        return;
    }

    let app = config.create_app();

    app.connect_startup(|_| load_css());
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

    // application.connect_startup(|_| {
    //     load_css();
    // });

    // application.connect_activate(|app| {
    //     let window = ApplicationWindow::builder()
    //         .application(app)
    //         .title("GTK Web Widget")
    //         .build();

    //     // Create a WebView
    //     let web_view = WebView::new();

    //     // Enable developer extras (web inspector)
    //     let settings = webkit6::prelude::WebViewExt::settings(&web_view).unwrap();
    //     settings.set_enable_developer_extras(true);

    //     // Start a simple HTTP server in a separate thread
    //     let current_dir = std::env::current_dir().unwrap();
    //     let dist_path = current_dir.parent().unwrap().join("dist");
    //     let dist_path_clone = dist_path.clone();

    //     thread::spawn(move || {
    //         start_simple_server(dist_path_clone, 8080);
    //     });

    //     // Give the server a moment to start
    //     thread::sleep(Duration::from_millis(100));

    //     // Load from the local HTTP server
    //     web_view.load_uri("http://localhost:8080");
    //     web_view.set_background_color(&RGBA::new(0.0, 0.0, 0.0, 0.0));

    //     window.set_child(Some(&web_view));
    //     window.init_layer_shell();
    //     window.set_layer(Layer::Top);
    //     window.auto_exclusive_zone_enable();
    //     let mut font_options = FontOptions::new().unwrap();
    //     font_options.set_antialias(gdk::cairo::Antialias::Best);
    //     window.set_font_options(Some(&font_options));

    //     if let Some(display) = Display::default() {
    //         let monitors = display.monitors();
    //         let monitor = monitors
    //             .item(0)
    //             .and_then(|obj| obj.downcast::<gtk4::gdk::Monitor>().ok());
    //         let d: &gtk4::gdk::Monitor = monitor.as_ref().unwrap();

    //         let width = d.geometry().width();

    //         window.set_width_request(width - 100);
    //         window.set_height_request(30);

    //         window.set_anchor(Edge::Top, true);
    //         window.set_anchor(Edge::Left, false);
    //         window.set_anchor(Edge::Right, false);
    //         window.set_anchor(Edge::Bottom, false);

    //         // Set window parameters
    //         // window.set_keep_above(true);
    //         window.set_resizable(false);
    //         window.set_decorated(false);
    //         window.set_tooltip_text(Some("Web Widget"));

    //         // Dock the window
    //         //window.set_type_hint(WindowTypeHint::Dock);

    //         // Show the gtk window
    //         window.present();
    //     }
    // });

    // application.run();
}

// fn start_simple_server(root_dir: std::path::PathBuf, port: u16) {
//     let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
//     println!("Server running on http://localhost:{}", port);

//     for stream in listener.incoming() {
//         match stream {
//             Ok(stream) => {
//                 handle_request(stream, &root_dir);
//             }
//             Err(e) => {
//                 eprintln!("Error: {}", e);
//             }
//         }
//     }
// }

// fn handle_request(mut stream: TcpStream, root_dir: &Path) {
//     use std::io::prelude::*;

//     let mut buffer = [0; 1024];
//     stream.read(&mut buffer).unwrap();

//     let request = String::from_utf8_lossy(&buffer[..]);
//     let request_line = request.lines().next().unwrap_or("");

//     // Parse the requested path
//     let path = if let Some(path) = request_line.split_whitespace().nth(1) {
//         if path == "/" {
//             "/index.html"
//         } else {
//             path
//         }
//     } else {
//         "/index.html"
//     };

//     let file_path = root_dir.join(&path[1..]); // Remove leading '/'

//     let (status, content_type, body) = if file_path.exists() && file_path.starts_with(root_dir) {
//         let content = fs::read(&file_path).unwrap_or_default();
//         let content_type = get_content_type(&file_path);
//         ("200 OK", content_type, content)
//     } else {
//         let content = b"404 Not Found".to_vec();
//         ("404 NOT FOUND", "text/plain", content)
//     };

//     let response = format!(
//         "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
//         status,
//         content_type,
//         body.len()
//     );

//     stream.write_all(response.as_bytes()).unwrap();
//     stream.write_all(&body).unwrap();
//     stream.flush().unwrap();
// }

// fn get_content_type(path: &Path) -> &'static str {
//     match path.extension().and_then(|s| s.to_str()) {
//         Some("html") => "text/html",
//         Some("css") => "text/css",
//         Some("js") => "application/javascript",
//         Some("json") => "application/json",
//         Some("png") => "image/png",
//         Some("jpg") | Some("jpeg") => "image/jpeg",
//         Some("gif") => "image/gif",
//         Some("svg") => "image/svg+xml",
//         Some("woff") => "font/woff",
//         Some("woff2") => "font/woff2",
//         _ => "text/plain",
//     }
// }
