mod server;
mod widget;
mod window;

use crate::widget::WidgetFactory;
use common::{config::Config, util};
use gtk4::gio::{ApplicationFlags, prelude::*};
use std::error::Error;

pub use server::start_server;

pub fn setup_widgets(config: Config) -> Result<(), Box<dyn Error>> {
    let app = gtk4::Application::new(Some("com.www.idgets"), ApplicationFlags::FLAGS_NONE);

    app.connect_activate(move |app| {
        util::disable_gtk_logs();

        let factory = WidgetFactory::new(app);
        let widgets = factory.create_widgets(&config);

        for widget in widgets {
            for window in widget.windows {
                window.show();
            }
        }
    });

    // Run without any args otherwise gtk will capture rust's args
    app.run_with_args(&[] as &[&str]);

    Ok(())
}
