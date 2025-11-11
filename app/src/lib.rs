mod server;
mod widget;
mod window;

use crate::{server::EventRequest, widget::WidgetFactory};
use common::{config::Config, util};
use gtk4::gio::{ApplicationFlags, prelude::*};
use std::{error::Error, rc::Rc};
use tokio::sync::mpsc::UnboundedReceiver;
use traccia::{debug, warn};

pub use server::start_server;

pub fn setup_widgets(
    config: Config,
    event_recv: UnboundedReceiver<EventRequest>,
) -> Result<(), Box<dyn Error>> {
    let app = gtk4::Application::new(Some("com.www.idgets"), ApplicationFlags::FLAGS_NONE);
    let event_recv = std::cell::RefCell::new(Some(event_recv));

    app.connect_activate(move |app| {
        util::disable_gtk_logs();

        let factory = WidgetFactory::new(app);
        let widgets = factory.create_widgets(&config, config.port);
        let widgets = Rc::new(widgets);

        if let Some(mut recv) = event_recv.borrow_mut().take() {
            let widgets = Rc::clone(&widgets);

            gtk4::glib::spawn_future_local(async move {
                while let Some(event) = recv.recv().await {
                    let mut count = 0;
                    let payload = Rc::new(event.payload);

                    for widget in widgets.iter() {
                        if widget.config.label == event.widget_label {
                            for window in &widget.windows {
                                window.dispatch(&event.event_name, payload.as_ref());
                                count += 1;
                            }
                        }
                    }

                    if count > 0 {
                        debug!(
                            "Dispatched event '{}' to {} window(s) of widget '{}'.",
                            event.event_name, count, event.widget_label
                        );
                    } else {
                        warn!(
                            "No windows found for widget '{}' to dispatch event '{}'.",
                            event.widget_label, event.event_name
                        );
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
