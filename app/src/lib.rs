mod server;
mod widget;
mod window;

use crate::{
    widget::WidgetFactory,
    window::{WindowAction, WindowActionRequest},
};
use common::{config::Config, util};
use gtk4::gio::{ApplicationFlags, prelude::*};
use std::{cell::RefCell, error::Error, rc::Rc};
use tokio::sync::mpsc::UnboundedReceiver;
use traccia::debug;

pub use server::start_server;

pub fn setup_widgets(
    config: Config,
    window_rx: UnboundedReceiver<WindowActionRequest>,
) -> Result<(), Box<dyn Error>> {
    let app = gtk4::Application::new(Some("com.www.idgets"), ApplicationFlags::FLAGS_NONE);
    let window_rx = RefCell::new(Some(window_rx));

    app.connect_activate(move |app| {
        util::disable_gtk_logs();

        let factory = WidgetFactory::new(app);
        let widgets = factory.create_widgets(&config, config.port);
        let widgets = Rc::new(widgets);

        if let Some(mut recv) = window_rx.borrow_mut().take() {
            let widgets = Rc::clone(&widgets);

            gtk4::glib::spawn_future_local(async move {
                while let Some(event) = recv.recv().await {
                    debug!("Handling window action: {:?}", event.action);

                    match event.action {
                        WindowAction::DispatchEvent(name, payload) => {
                            for widget in widgets.iter() {
                                if widget.config.label == event.target {
                                    for window in &widget.windows {
                                        window.dispatch(&name, &payload);
                                    }
                                }
                            }
                        }

                        WindowAction::Show => {
                            for widget in widgets.iter() {
                                if widget.config.label == event.target {
                                    for window in &widget.windows {
                                        window.show();
                                    }
                                }
                            }
                        }

                        WindowAction::Hide => {
                            for widget in widgets.iter() {
                                if widget.config.label == event.target {
                                    for window in &widget.windows {
                                        window.hide();
                                    }
                                }
                            }
                        }
                    }
                }
            });
        }

        for widget in widgets.iter() {
            for window in &widget.windows {
                window.show();
            }
        }
    });

    // Run without any args otherwise gtk will capture rust's args
    app.run_with_args(&[] as &[&str]);

    Ok(())
}
