use gtk4::prelude::*;
use std::cell::Cell;
use traccia::debug;
use webkit6::WebView;

#[derive(Debug, Clone)]
pub enum WindowAction {
    Show,
    Hide,
}

impl From<String> for WindowAction {
    fn from(value: String) -> Self {
        match value.as_str() {
            "show" => WindowAction::Show,
            "hide" => WindowAction::Hide,
            _ => panic!("This should never happen"),
        }
    }
}

/// This payload is used as the body of a request from
/// the client to perform something on a window.
///
/// This includes a target widget because
/// you could, for example, show a modal widget by clicking on
/// another's widget button.
#[derive(Debug, Clone)]
pub struct WindowActionRequest {
    pub target_label: String,
    pub action: WindowAction,
}

pub struct Window {
    pub gtk_window: gtk4::ApplicationWindow,
    #[allow(unused)]
    pub webview: WebView,
    pub id: u32,
    #[allow(unused)]
    pub monitor_id: String,
    is_visible: Cell<bool>,
}

impl Window {
    pub fn new(
        gtk_window: gtk4::ApplicationWindow,
        webview: WebView,
        id: u32,
        monitor_id: String,
        initial_visibility: bool,
    ) -> Self {
        Self {
            gtk_window,
            webview,
            id,
            monitor_id,
            is_visible: Cell::new(initial_visibility),
        }
    }

    pub fn show(&self) {
        if !self.is_visible.get() {
            debug!("Showing window {}", self.id);
            self.gtk_window.show();
            self.is_visible.set(true);
        }
    }

    pub fn hide(&self) {
        if self.is_visible.get() {
            debug!("Hiding window {}", self.id);
            self.gtk_window.hide();
            self.is_visible.set(false);
        }
    }
}
