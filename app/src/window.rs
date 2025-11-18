use gtk4::prelude::*;
use traccia::{debug, error};
use webkit6::{WebView, prelude::WebViewExt};

#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct WindowActionRequest {
    pub target: String,
    pub action: WindowAction,
}

pub struct Window {
    pub gtk_window: gtk4::ApplicationWindow,
    pub webview: WebView,
    pub id: u32,
    #[allow(unused)]
    pub monitor_id: String,
}

impl Window {
    pub fn show(&self) {
        debug!("Showing window {}", self.id);
        self.gtk_window.show();
        self.gtk_window.present();
    }

    pub fn hide(&self) {
        debug!("Hiding window {}", self.id);
        self.gtk_window.hide();
    }
}
