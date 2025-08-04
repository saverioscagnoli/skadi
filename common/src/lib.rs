use std::sync::OnceLock;

pub mod config;
pub mod io;

static DEBUG: OnceLock<bool> = OnceLock::new();
static DEV: OnceLock<bool> = OnceLock::new();

pub fn set_debug_mode(debug: bool) {
    DEBUG.set(debug).unwrap_or_else(|_| {
        panic!("DEBUG has already been set to {}", debug);
    });
}

pub fn set_dev_mode(dev: bool) {
    DEV.set(dev).unwrap_or_else(|_| {
        panic!("DEV has already been set to {}", dev);
    });
}

pub fn debug_mode() -> bool {
    cfg!(debug_assertions) || DEBUG.get().copied().unwrap_or(false)
}

pub fn dev_mode() -> bool {
    DEV.get().copied().unwrap_or(false)
}
