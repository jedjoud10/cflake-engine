use log::Level;
use parking_lot::Mutex;
use std::sync::Arc;

// Logger that implements the log trait
pub struct Logger {
    // We keep the log messages in memory, when we exit the game we write them all into the file
    pub(crate) messages: Arc<Mutex<Vec<String>>>,
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            // Damn okay the log crate kinda do be bopping
            let src_file = record.file().unwrap();
            let line = record.line().unwrap();
            let level = record.level();
            let args = record.args();
            let mut mutex = self.messages.lock();
            let string = format!(
                "[{src};{line}][{level}]: {msg}",
                src = src_file,
                line = line,
                level = level,
                msg = args
            );
            mutex.push(string);
        }
    }

    fn flush(&self) {}
}
