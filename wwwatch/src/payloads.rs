use crate::Op;
use serde::Serialize;

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
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub read: u64,
    pub write: u64,
    pub name: String,
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
