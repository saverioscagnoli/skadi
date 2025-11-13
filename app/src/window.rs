use gtk4::prelude::*;
use webkit6::{WebView, prelude::WebViewExt};

pub struct Window {
    pub gtk_window: gtk4::ApplicationWindow,
    pub webview: WebView,
    pub id: u32,
    pub monitor_id: String,
}

impl Window {
    pub fn show(&self) {
        self.gtk_window.show();
        self.gtk_window.present();
    }

    pub fn dispatch<N: AsRef<str>>(&self, name: N, event: &str) {
        let script = format!(
            "window.dispatchEvent(new CustomEvent('{}', {{ detail: {} }}));",
            name.as_ref(),
            event,
        );

        self.webview.evaluate_javascript(
            &script,
            None,
            None,
            None::<&gtk4::gio::Cancellable>,
            |_| {},
        );
    }
}
