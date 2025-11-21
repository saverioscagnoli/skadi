use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OpCode {
    Workspaces,
    Info,
    Notification,
}

pub trait SerializePrint: Serialize {
    fn print(&self) {
        println!("{}", serde_json::to_string(&self).expect(":3"));
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Payload<T> {
    pub op: OpCode,
    pub data: T,
}

impl<T: Serialize> SerializePrint for Payload<T> {}

#[derive(Debug, Serialize)]
pub struct WorkspacesPayload {
    pub focused: i32,
    pub workspaces: Vec<(i32, String)>,
}

#[derive(Debug, Serialize)]
pub struct CoreInfo {
    pub usage: f32,
    pub freq: u64,
}

#[derive(Debug, Serialize)]
pub struct CpuInfo {
    pub usage: f32,
    pub temp: f32,
    pub freq: u64,
    pub cores: Vec<CoreInfo>,
}

#[derive(Debug, Serialize)]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
}

#[derive(Debug, Serialize)]
pub struct DiskInfo {
    pub primary: bool,
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub read: u64,
    pub write: u64,
    pub name: String,
    pub mountpoint: String,
}

#[derive(Debug, Serialize)]
pub struct InterfaceInfo {
    pub primary: bool,
    pub name: String,
    pub download: u64,
    pub upload: u64,
}

#[derive(Debug, Serialize)]
pub struct NetworkInfo {
    pub download: u64,
    pub upload: u64,
    pub interfaces: Vec<InterfaceInfo>,
}

#[derive(Debug, Serialize)]
pub struct InfoPayload {
    pub cpu: Option<CpuInfo>,
    pub memory: Option<MemoryInfo>,
    pub disks: Option<Vec<DiskInfo>>,
    pub network: Option<NetworkInfo>,
}

/// https://specifications.freedesktop.org/notification/latest/basic-design.html
#[derive(Debug, Clone, Serialize)]
pub struct NotificationPayload {
    pub app_name: String,
    /// An optional ID of an existing notification that this notification is intended to replace.
    pub replaces_id: u32,
    pub notification_icon: Option<String>,
    /// Path to the image file.
    pub image: Option<PathBuf>,
    /// This is a single line overview of the notification. For instance,
    /// "You have mail" or "A friend has come online".
    /// It should generally not be longer than 40 characters, though this is not a requirement,
    /// and server implementations should word wrap if necessary.
    /// The summary must be encoded using UTF-8.
    pub summary: String,
    /// This is a multi-line body of text. Each line is a paragraph,
    /// server implementations are free to word wrap them as they see fit.
    /// The body may contain simple markup as specified in Markup.
    /// It must be encoded using UTF-8.
    /// If the body is omitted, just the summary is displayed.
    pub body: String,
    pub actions: Vec<String>,
    /// The timeout time in milliseconds since the display of the notification at which
    /// the notification should automatically close.
    /// If -1, the notification's expiration time is dependent
    /// on the notification server's settings, and may vary for the type of notification.
    /// If 0, the notification never expires.
    pub expiration: i32,
}
