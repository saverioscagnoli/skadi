use gtk4::prelude::*;
use serde::Serialize;
use webkit6::{WebView, prelude::WebViewExt};

pub struct Window {
    pub gtk_window: gtk4::ApplicationWindow,
    pub webview: WebView,
    pub id: u32,
    pub monitor_id: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum WindowAction {
    Show,
    Hide,
}

/// This payload is used as the body of a request from
/// the client to perform something on a window.
///
/// This includes a target widget because
/// you could, for example, show a modal widget by clicking on
/// another's widget button.
#[derive(Debug, Clone, Serialize)]
pub struct WindowActionRequest {
    pub requested_by: String,
    pub target: String,
    pub action: WindowAction,
}

impl Window {
    pub fn show(&self) {
        self.gtk_window.show();
        self.gtk_window.present();
    }

    pub fn hide(&self) {
        self.gtk_window.hide();
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
