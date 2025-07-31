use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct Spinner {
    frames: Vec<&'static str>,
    message: String,
    delay: Duration,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            frames: vec!["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
            message: String::new(),
            delay: Duration::from_millis(100),
        }
    }

    pub fn with_message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    pub fn start(&self) -> SpinnerHandle {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        let frames = self.frames.clone();
        let message = Arc::new(Mutex::new(self.message.clone()));
        let message_clone = message.clone();
        let delay = self.delay;

        let handle = thread::spawn(move || {
            let mut frame_index = 0;

            // Hide cursor
            io::stdout().flush().unwrap();

            while running_clone.load(Ordering::Relaxed) {
                // Clear the line and move cursor to beginning
                print!("\r\x1B[K");

                // Get current message
                let current_message = message_clone.lock().unwrap().clone();

                // Print spinner frame and message
                print!("{} {}", frames[frame_index], current_message);
                io::stdout().flush().unwrap();

                frame_index = (frame_index + 1) % frames.len();
                thread::sleep(delay);
            }

            // Clear the line and show cursor again
            print!("\r\x1B[K");
            io::stdout().flush().unwrap();
        });

        SpinnerHandle {
            running,
            message,
            handle: Some(handle),
        }
    }
}

pub struct SpinnerHandle {
    running: Arc<AtomicBool>,
    message: Arc<Mutex<String>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl SpinnerHandle {
    pub fn update_message(&self, new_message: &str) {
        if let Ok(mut message) = self.message.lock() {
            *message = new_message.to_string();
        }
    }

    pub fn finish_with_symbol_and_message(&mut self, symbol: &str, message: &str) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle.join().unwrap();
        }
        println!("{} {}", symbol, message);
    }
}

impl Drop for SpinnerHandle {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
