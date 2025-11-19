mod server;
mod widget;
mod window;

use crate::{
    widget::WidgetFactory,
    window::{WindowAction, WindowActionRequest},
};
use common::{config::Config, util};
use gtk4::gio::{ApplicationFlags, prelude::*};
use std::{cell::RefCell, collections::HashMap, error::Error, rc::Rc, time::Duration};
use tokio::sync::mpsc::UnboundedReceiver;
use traccia::{debug, warn};

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
        let mut widget_map: HashMap<String, Vec<usize>> = HashMap::new();

        for (idx, widget) in widgets.iter().enumerate() {
            widget_map.insert(widget.config.label.clone(), vec![idx]);
        }

        let widgets = Rc::new(widgets);
        let widget_map = Rc::new(widget_map);

        if let Some(mut recv) = window_rx.borrow_mut().take() {
            let widgets = Rc::clone(&widgets);
            let widget_map = Rc::clone(&widget_map);

            gtk4::glib::spawn_future_local(async move {
                while let Some(event) = recv.recv().await {
                    debug!("Handling window action: {:?}", event.action);

                    if let Some(indices) = widget_map.get(&event.target_label) {
                        for &idx in indices {
                            let widget = &widgets[idx];

                            if widget.windows.is_empty() {
                                continue;
                            }

                            match event.action {
                                WindowAction::Show => {
                                    for window in &widget.windows {
                                        window.show();
                                    }
                                }
                                WindowAction::Hide => {
                                    for window in &widget.windows {
                                        window.hide();
                                    }
                                }
                            }
                        }
                    } else {
                        debug!("Widget with label '{}' not found", event.target_label);
                    }
                }

                warn!("Receiver stopped. wtf?");
            });
        }
    });

    // Run without any args otherwise gtk will capture rust's args
    app.run_with_args(&[] as &[&str]);

    Ok(())
}
