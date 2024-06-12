use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc,Mutex};
use lazy_static::lazy_static;

lazy_static! {
    static ref LOGGER: Arc<Mutex<Logger>> = Arc::new(Mutex::new(Logger::new("log.txt")));
}

struct Logger {
    filename: String,
}

impl Logger {
    fn new(filename: &str) -> Self {
        Logger {
            filename: filename.to_string(),
        }
    }

    fn log(&self, message: &str) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.filename)
            .expect("Unable to open log file");

        writeln!(file, "{}", message).expect("Unable to write to log file");
    }
}

pub fn log_message(message: &str) {
    if let Ok(logger) = LOGGER.lock() {
        logger.log(message);
    };
}

