use gtk4::prelude::*;

pub struct Window {
    pub gtk_window: gtk4::ApplicationWindow,
    pub id: u32,
    pub monitor_id: String,
}

impl Window {
    pub fn show(&self) {
        self.gtk_window.show();
        self.gtk_window.present();
    }
}
