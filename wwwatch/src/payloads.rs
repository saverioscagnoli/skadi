use crate::Op;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
pub struct WorkspacePayload<'a> {
    pub op: Op,
    pub current: &'a String,
    pub total: &'a Vec<(i32, String)>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CoreInfo {
    pub usage: f32,
    pub freq: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CpuPayload {
    pub usage: f32,
    pub temp: f32,
    pub freq: f32,
    pub cores: Vec<CoreInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MemPayload {
    pub total: u64,
    pub used: u64,
    pub free: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskInfo {
    pub primary: bool,
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub read: u64,
    pub write: u64,
    pub name: String,
    pub mount_point: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiskPayload {
    pub disks: Vec<DiskInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkInterface {
    pub name: String,
    pub download: u64,
    pub upload: u64,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkPayload {
    pub download: u64,
    pub upload: u64,
    pub interfaces: Vec<NetworkInterface>,
}

#[derive(Debug, Clone, Serialize)]
pub struct InfoPayload {
    pub op: Op,
    pub cpu: Option<CpuPayload>,
    pub mem: Option<MemPayload>,
    pub disks: Option<DiskPayload>,
    pub network: Option<NetworkPayload>,
}

/// https://specifications.freedesktop.org/notification/latest/basic-design.html
#[derive(Debug, Clone, Serialize)]
pub struct NotificationPayload {
    pub op: Op,
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
