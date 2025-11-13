use crate::payloads::{
    CoreInfo, CpuPayload, DiskInfo, DiskPayload, InfoPayload, MemPayload, NetworkInterface,
    NetworkPayload,
};
use std::{thread, time::Duration};
use sysinfo::{
    Components, CpuRefreshKind, DiskRefreshKind, Disks, MemoryRefreshKind, NetworkData, Networks,
    RefreshKind, System,
};

fn get_cpu_temp() -> Option<f32> {
    // Iterate through all hwmon devices
    for entry in std::fs::read_dir("/sys/class/hwmon").ok()? {
        let hwmon = entry.ok()?.path();

        // Try to read labels and find something that looks like CPU
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
                    // Read temperature value
                    if let Ok(raw) = std::fs::read_to_string(&temp_path) {
                        if let Ok(val) = raw.trim().parse::<f32>() {
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
    }

    None
}

fn get_primary_network(networks: &Networks) -> Option<(&String, &NetworkData)> {
    networks
        .iter()
        .filter(|(name, _)| !name.starts_with("lo"))
        .max_by_key(|(_, data)| data.received() + data.transmitted())
}

pub fn query_info(cpu: bool, mem: bool, disk: bool, network: bool, interval_ms: u64) {
    if !cpu && !mem && !disk && !network {
        return;
    }

    let mut refresh_kind = RefreshKind::nothing();
    let mut disk_refresh_kind = DiskRefreshKind::nothing();

    if cpu {
        refresh_kind = refresh_kind.with_cpu(CpuRefreshKind::everything())
    }

    if mem {
        refresh_kind = refresh_kind.with_memory(MemoryRefreshKind::everything());
    }

    if disk {
        disk_refresh_kind = DiskRefreshKind::everything();
    }

    let mut info_payload = InfoPayload::default();
    let mut system = System::new_with_specifics(refresh_kind);
    let mut components = Components::new_with_refreshed_list();
    let disks = Disks::new_with_refreshed_list_specifics(disk_refresh_kind);
    let mut networks = Networks::new_with_refreshed_list();

    thread::spawn(move || {
        // Initial refresh to establish baseline for CPU usage calculations
        system.refresh_specifics(refresh_kind);
        thread::sleep(Duration::from_millis(100));

        loop {
            system.refresh_specifics(refresh_kind);
            components.refresh(false);
            networks.refresh(false);

            if cpu {
                let mut avg_freq = 0.0;
                let mut cores = Vec::new();

                // Get CPU data after refresh
                let cpus = system.cpus();

                for cpu in cpus {
                    let usage = cpu.cpu_usage();
                    let freq = cpu.frequency() as f32;

                    avg_freq += freq;

                    cores.push(CoreInfo { usage, freq });
                }

                avg_freq = avg_freq / cpus.len() as f32;

                let payload = CpuPayload {
                    usage: system.global_cpu_usage(),
                    temp: get_cpu_temp().unwrap_or(-1.0),
                    freq: avg_freq,
                    cores,
                };

                info_payload.cpu = Some(payload);
            }

            if mem {
                let payload = MemPayload {
                    total: system.total_memory(),
                    used: system.used_memory(),
                    free: system.available_memory(),
                };

                info_payload.mem = Some(payload);
            }

            if disk {
                let mut disks_payload = Vec::new();

                for disk in &disks {
                    let total = disk.total_space();
                    let free = disk.available_space();
                    let used = total - free;
                    let usage = disk.usage();

                    disks_payload.push(DiskInfo {
                        total,
                        used,
                        free,
                        read: usage.total_read_bytes,
                        write: usage.total_written_bytes,
                        name: disk.name().to_string_lossy().to_string(),
                    });
                }

                let payload = DiskPayload {
                    disks: disks_payload,
                };

                info_payload.disks = Some(payload);
            }

            if network {
                let mut networks_payload = Vec::new();
                let mut total_bytes_download = 0;
                let mut total_bytes_upload = 0;

                let primary_interface = get_primary_network(&networks);

                for (name, data) in &networks {
                    total_bytes_download += data.received();
                    total_bytes_upload += data.transmitted();

                    networks_payload.push(NetworkInterface {
                        name: name.to_string(),
                        download: data.received(),
                        upload: data.transmitted(),
                        primary: primary_interface.map(|(n, _)| n == name).unwrap_or(false),
                    });
                }

                let payload = NetworkPayload {
                    download: total_bytes_download,
                    upload: total_bytes_upload,
                    interfaces: networks_payload,
                };

                info_payload.network = Some(payload);
            }

            println!("{}", serde_json::to_string(&info_payload).unwrap());
            thread::sleep(Duration::from_millis(interval_ms));
        }
    });
}
