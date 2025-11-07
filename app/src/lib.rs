mod widget;

use crate::widget::Widget;
use common::{config::Config, util};
use gtk4::gio::{ApplicationFlags, prelude::*};
use std::error::Error;
use traccia::error;

pub fn start() -> Result<(), Box<dyn Error>> {
    let config = Config::parse()?;
    let app = gtk4::Application::new(Some("com.www.idgets"), ApplicationFlags::FLAGS_NONE);

    app.connect_activate(move |app| {
        util::disable_gtk_logs();

        for wc in config.windows.iter() {
            let widget = Widget::new(&app, wc.clone());

            if let Err(e) = widget.init() {
                error!("Failed to initialize layer shell: {}", e);
            }
        }
    });

    app.run();

    Ok(())
}
