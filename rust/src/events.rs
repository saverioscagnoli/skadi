use crate::error;
use std::{collections::HashMap, ops::Deref, sync::Arc};
use tokio::sync::Mutex;
use traccia::error;
use webkit6::prelude::WebViewExt;

#[cfg(debug_assertions)]
use traccia::debug;

#[derive(Debug, Clone)]
pub struct SendWebView {
    pub webview: webkit6::WebView,
}

/// I dont care, im a freaky frog
unsafe impl Send for SendWebView {}
unsafe impl Sync for SendWebView {}

impl Deref for SendWebView {
    type Target = webkit6::WebView;

    fn deref(&self) -> &Self::Target {
        &self.webview
    }
}

pub trait JsEventEmitter {
    fn emit_to_js(&self, name: &str, payload: serde_json::Value);
}

impl JsEventEmitter for webkit6::WebView {
    fn emit_to_js(&self, name: &str, payload: serde_json::Value) {
        let script = format!(
            "window.dispatchEvent(new CustomEvent('{}', {{ detail: {} }}));",
            name,
            payload.to_string()
        );

        self.evaluate_javascript(
            &script,
            None::<&str>,
            None,
            None::<&gtk4::gio::Cancellable>,
            |_| {},
        );
    }
}

#[derive(Clone)]
pub struct EventEmitter {
    webviews: Arc<Mutex<HashMap<String, SendWebView>>>,
}

impl EventEmitter {
    pub fn new(webviews: Arc<Mutex<HashMap<String, SendWebView>>>) -> Self {
        Self { webviews }
    }

    pub async fn add_webview(&self, label: String, webview: SendWebView) {
        let mut webviews = self.webviews.lock().await;
        webviews.insert(label, webview);
    }

    pub async fn emit_to(&self, label: &str, name: &str, payload: serde_json::Value) {
        let webviews = self.webviews.lock().await;

        if let Some(webview) = webviews.get(label) {
            let main_conext = gtk4::glib::MainContext::default();
            let webview = webview.clone();
            let name = name.to_string();

            #[cfg(debug_assertions)]
            debug!(
                "Emitting event '{}' to webview with label '{}'",
                name, label
            );

            main_conext.invoke(move || {
                webview.emit_to_js(&name, payload);
            });
        } else {
            error!("Webview with label '{}' not found", label);
        }
    }

    // pub async fn emit(&self, name: &str, payload: serde_json::Value) {
    //     let webviews = self.webviews.lock().await;
    //     let name = name.to_string();

    //     for webview in webviews.values() {
    //         let webview_clone = webview.clone();
    //         let payload_clone = payload.clone();
    //         let name_clone = name.clone();

    //         let main_context = gtk4::glib::MainContext::default();
    //         main_context.invoke(move || {
    //             webview_clone.emit_to_js(&name_clone, payload_clone);
    //         });
    //     }
    // }
}
