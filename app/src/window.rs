use gtk4::prelude::*;
use std::fmt;
use webkit6::{WebView, prelude::WebViewExt};

type EventName = String;
/// Event data is just the raw standard output from the process.
/// I left it as a string because I want the burden of serialization to be on the client.
/// This allows the client to choose the format of the data.
///
/// Is this a stupid idea?
type EventData = String;

#[derive(Clone)]
pub enum WindowAction {
    DispatchEvent(EventName, EventData),
    Show,
    Hide,
}

impl fmt::Debug for WindowAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WindowAction::DispatchEvent(name, data) => {
                let truncated = if data.len() > 100 {
                    format!("{}... ({} bytes)", &data[..100], data.len())
                } else {
                    data.clone()
                };
                f.debug_tuple("DispatchEvent")
                    .field(name)
                    .field(&truncated)
                    .finish()
            }
            WindowAction::Show => write!(f, "Show"),
            WindowAction::Hide => write!(f, "Hide"),
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
    pub target: String,
    pub action: WindowAction,
}

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

    pub fn hide(&self) {
        self.gtk_window.hide();
    }

    pub fn dispatch<N: AsRef<str>, P: AsRef<str>>(&self, name: N, payload: P) {
        let script = format!(
            "window.dispatchEvent(new CustomEvent('{}', {{ detail: {} }}));",
            name.as_ref(),
            payload.as_ref(),
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
