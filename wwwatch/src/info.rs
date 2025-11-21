use crate::{
    InfoArgs,
    payload::{
        CoreInfo, CpuInfo, DiskInfo, InfoPayload, InterfaceInfo, MemoryInfo, NetworkInfo, OpCode,
        Payload, SerializePrint,
    },
};
use std::{path::Path, time::Duration};
use sysinfo::{
    Components, CpuRefreshKind, DiskRefreshKind, Disks, MemoryRefreshKind, NetworkData, Networks,
    RefreshKind, System,
};

fn get_cpu_temp() -> Option<f32> {
    // Iterate through all hwmon devices
    for entry in std::fs::read_dir("/sys/class/hwmon").ok()? {
        let hwmon = entry.ok()?.path();

        for temp_entry in std::fs::read_dir(&hwmon).ok()? {
            let temp_path = temp_entry.ok()?.path();

            if temp_path.file_name()?.to_string_lossy().starts_with("temp")
                && temp_path.to_string_lossy().contains("_input")
            {
                let label_path = temp_path.with_file_name(
                    temp_path
                        .file_name()?
                        .to_string_lossy()
                        .replace("_input", "_label"),
                );

                let label = std::fs::read_to_string(&label_path)
                    .unwrap_or_default()
                    .to_lowercase();

                if label.contains("cpu") || label.contains("core") || label.contains("k10temp") {
                    if let Ok(raw) = std::fs::read_to_string(&temp_path)
                        && let Ok(val) = raw.trim().parse::<f32>()
                    {
                        // Usually in millidegrees
                        if val > 1000.0 {
                            return Some(val / 1000.0);
                        } else {
                            return Some(val);
                        }
                    }
                }
            }
        }
    }

    None
}

fn get_primary_network(networks: &Networks) -> Option<(&String, &NetworkData)> {
    networks
        .iter()
        .filter(|(name, _)| !name.starts_with("lo"))
        .max_by_key(|(_, data)| data.received() + data.transmitted())
}

pub async fn poll(info: &InfoArgs) {
    if !info.cpu && !info.mem && !info.disks && !info.net {
        return;
    }

    let mut refresh_kind = RefreshKind::nothing();
    let mut disk_refresh_kind = DiskRefreshKind::nothing();

    if info.cpu {
        refresh_kind = refresh_kind.with_cpu(CpuRefreshKind::everything());
    }

    if info.mem {
        refresh_kind = refresh_kind.with_memory(MemoryRefreshKind::everything());
    }

    if info.disks {
        disk_refresh_kind = DiskRefreshKind::everything();
    }

    let mut system = System::new_with_specifics(refresh_kind);
    let mut components = Components::new_with_refreshed_list();
    let mut disks = Disks::new_with_refreshed_list_specifics(disk_refresh_kind);
    let mut networks = Networks::new_with_refreshed_list();
    let mut payload = Payload {
        op: OpCode::Info,
        data: InfoPayload {
            cpu: None,
            memory: None,
            disks: None,
            network: None,
        },
    };

    system.refresh_specifics(refresh_kind);
    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut interval = tokio::time::interval(Duration::from_millis(info.interval));

    loop {
        system.refresh_specifics(refresh_kind);
        components.refresh(true);
        disks.refresh(true);
        networks.refresh(true);

        if info.cpu {
            let mut avg_usage = 0.0;
            let mut avg_freq = 0;
            let mut cores = Vec::new();
            let cpus = system.cpus();

            for cpu in cpus {
                let usage = cpu.cpu_usage();
                let freq = cpu.frequency();

                avg_usage += usage;
                avg_freq += freq;
                cores.push(CoreInfo { usage, freq });
            }

            avg_usage /= cpus.len() as f32;
            avg_freq /= cpus.len() as u64;

            let cpu_info = CpuInfo {
                usage: avg_usage,
                temp: get_cpu_temp().unwrap_or(-1.0),
                freq: avg_freq,
                cores,
            };

            payload.data.cpu = Some(cpu_info);
        }

        if info.mem {
            let mem_info = MemoryInfo {
                total: system.total_memory(),
                used: system.used_memory(),
                free: system.free_memory(),
            };

            payload.data.memory = Some(mem_info);
        }

        if info.disks {
            let mut disks_payload = Vec::new();

            for disk in &disks {
                let total = disk.total_space();
                let free = disk.available_space();
                let used = total - free;
                let usage = disk.usage();
                let primary = Path::new("/").starts_with(disk.mount_point());

                disks_payload.push(DiskInfo {
                    primary,
                    total,
                    used,
                    free,
                    read: usage.total_read_bytes,
                    write: usage.total_written_bytes,
                    name: disk.name().to_string_lossy().to_string(),
                    mountpoint: disk.mount_point().to_string_lossy().to_string(),
                });
            }

            payload.data.disks = Some(disks_payload);
        }

        if info.net {
            let mut interfaces = Vec::new();
            let mut total_bytes_download = 0;
            let mut total_bytes_upload = 0;

            let primary_interface = get_primary_network(&networks);

            for (name, data) in &networks {
                total_bytes_download += data.received();
                total_bytes_upload += data.transmitted();

                interfaces.push(InterfaceInfo {
                    primary: primary_interface.map(|(n, _)| n == name).unwrap_or(false),
                    name: name.to_string(),
                    download: data.received(),
                    upload: data.transmitted(),
                })
            }

            let network_info = NetworkInfo {
                download: total_bytes_download,
                upload: total_bytes_upload,
                interfaces,
            };

            payload.data.network = Some(network_info);
        }

        payload.print();
        interval.tick().await;
    }
}
