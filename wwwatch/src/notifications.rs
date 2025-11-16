use crate::{Op, payloads::NotificationPayload};
use common::{paths, util};
use image::{ImageBuffer, RgbaImage};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::{
    collections::HashMap,
    error::Error,
    fs,
    future::pending,
    path::{Path, PathBuf},
};
use zbus::{connection, interface, zvariant::Value};

fn extract_image(
    hints: &HashMap<String, Value>,
    cache: &mut HashMap<String, PathBuf>,
) -> Option<PathBuf> {
    // First, try to create a cache key from the image data
    for key in ["image-data", "image_data", "icon_data"] {
        if let Some(value) = hints.get(key) {
            if let Value::Structure(s) = value {
                let fields = s.fields();

                let width = if let Some(Value::I32(w)) = fields.get(0) {
                    *w as u32
                } else {
                    continue;
                };

                let height = if let Some(Value::I32(h)) = fields.get(1) {
                    *h as u32
                } else {
                    continue;
                };

                let has_alpha = if let Some(Value::Bool(a)) = fields.get(3) {
                    *a
                } else {
                    true
                };

                if let Some(Value::Array(arr)) = fields.get(6) {
                    let bytes: Vec<u8> = arr
                        .iter()
                        .filter_map(|v| if let Value::U8(b) = v { Some(*b) } else { None })
                        .collect();

                    // Create a cache key from image dimensions and hash of data

                    let mut hasher = DefaultHasher::new();

                    width.hash(&mut hasher);
                    height.hash(&mut hasher);
                    bytes.hash(&mut hasher);

                    let cache_key = format!("image-data-{}", hasher.finish());

                    // Check if we already have this image cached
                    if let Some(cached_path) = cache.get(&cache_key) {
                        if cached_path.exists() {
                            return Some(cached_path.clone());
                        } else {
                            // Remove stale cache entry
                            cache.remove(&cache_key);
                        }
                    }

                    // Save new image
                    let file_name =
                        paths::tmp_dir().join(format!("{}.png", util::random_string(7)));

                    if save_raw_as_png(&file_name, width, height, &bytes, has_alpha).is_ok() {
                        let path = PathBuf::from(file_name);

                        cache.insert(cache_key, path.clone());

                        return Some(path);
                    }
                }
            }
        }
    }

    // Check for image-path
    for key in ["image-path", "image_path"] {
        if let Some(Value::Str(path)) = hints.get(key) {
            let path_str = path.as_str();

            // Use the original path as cache key
            let cache_key = format!("image-path-{}", path_str);

            // Check cache first
            if let Some(cached_path) = cache.get(&cache_key) {
                if cached_path.exists() {
                    return Some(cached_path.clone());
                } else {
                    cache.remove(&cache_key);
                }
            }

            // If it's already a file path, copy it to /tmp/wwwidgets
            if let Ok(bytes) = fs::read(path_str) {
                let ext = Path::new(path_str)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("png");

                let file_name =
                    paths::tmp_dir().join(format!("{}.{}", util::random_string(7), ext));

                if fs::write(&file_name, bytes).is_ok() {
                    let path = PathBuf::from(file_name);

                    cache.insert(cache_key, path.clone());

                    return Some(path);
                }
            }
        }
    }

    None
}

fn save_raw_as_png<P: AsRef<Path>>(
    file_name: P,
    width: u32,
    height: u32,
    data: &[u8],
    has_alpha: bool,
) -> Result<(), Box<dyn Error>> {
    let img: RgbaImage = if has_alpha {
        ImageBuffer::from_raw(width, height, data.to_vec())
            .ok_or("Failed to create image buffer")?
    } else {
        let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);

        for chunk in data.chunks(3) {
            rgba_data.extend_from_slice(chunk);
            rgba_data.push(255); // Add alpha
        }

        ImageBuffer::from_raw(width, height, rgba_data).ok_or("Failed to create image buffer")?
    };

    img.save(file_name)?;

    Ok(())
}

struct Notifications {
    image_cache: HashMap<String, PathBuf>,
}

#[interface(name = "org.freedesktop.Notifications")]
impl Notifications {
    fn notify(
        &mut self,
        app_name: &str,
        replaces_id: u32,
        notification_icon: &str,
        summary: &str,
        body: &str,
        actions: Vec<String>,
        hints: HashMap<String, Value<'_>>,
        expire_timeout: i32,
    ) -> u32 {
        let payload = NotificationPayload {
            op: Op::Notification,
            app_name: app_name.to_owned(),
            replaces_id,
            notification_icon: Some(notification_icon.to_owned()),
            image: extract_image(&hints, &mut self.image_cache),
            summary: summary.to_owned(),
            body: body.to_owned(),
            actions,
            expiration: expire_timeout,
        };

        println!("{}", serde_json::to_string(&payload).unwrap());

        1
    }

    fn get_server_information(&self) -> (&str, &str) {
        (
            "wwwatch",   // name
            "wwwidgets", // vendor
        )
    }

    fn get_capabilities(&self) -> Vec<&str> {
        vec![
            "body",
            "body-markup",
            "icon-static",
            "actions",
            "hints",
            "persistence",
        ]
    }

    fn close_notification(&mut self, _id: u32) {}
}

pub async fn listen() -> Result<(), Box<dyn Error>> {
    let _conn = connection::Builder::session()?
        .name("org.freedesktop.Notifications")?
        .serve_at(
            "/org/freedesktop/Notifications",
            Notifications {
                image_cache: HashMap::new(),
            },
        )?
        .build()
        .await?;

    pending::<()>().await;

    Ok(())
}
