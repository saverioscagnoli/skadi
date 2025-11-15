use std::{collections::HashMap, error::Error, future::pending};

use zbus::{connection, interface, zvariant::Value};

struct Notifications;

#[interface(name = "org.freedesktop.Notifications")]
impl Notifications {
    fn notify(
        &mut self,
        app_name: &str,
        replaces_id: u32,
        app_icon: &str,
        summary: &str,
        body: &str,
        actions: Vec<String>,
        hints: HashMap<String, Value<'_>>,

        expire_timeout: i32,
    ) -> u32 {
        println!("{} {}", app_name, summary);
        1
    }

    fn get_server_information(&self) -> (&str, &str, &str, &str) {
        (
            "wwwatch",   // name
            "wwwidgets", // vendor
            "0.1.0",     // version
            "1.2",       // spec_version
        )
    }

    fn get_capabilities(&self) -> Vec<&str> {
        vec!["body"]
    }

    fn close_notification(&mut self, id: u32) {
        println!("Closing notification {}", id);
    }
}

pub async fn check() -> Result<(), Box<dyn Error>> {
    let _conn = connection::Builder::session()?
        .name("org.freedesktop.Notifications")?
        .serve_at("/org/freedesktop/Notifications", Notifications)?
        .build()
        .await?;

    pending::<()>().await;

    Ok(())
}
