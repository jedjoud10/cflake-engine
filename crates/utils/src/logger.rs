use std::{cell::RefCell, path::PathBuf};

use log::{Level, Log};
use parking_lot::Mutex;

/*
FIXME
// This will log debug messages, warning messages, and errors to a file
// This will also print the messages to the console
pub struct Logger {
    level: Level,
    file: PathBuf,
    messages: Mutex<Vec<String>>,
}

impl Logger {
    // Create a new logger that will write to the most recent file
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.flush();
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let src_file = record.file().unwrap();
            let line = record.line().unwrap();
            let level = record.level();
            let args = record.args();
            let mut mutex = self.messages.lock();
            let string = format!("[{src};{line}][{level}]: {msg}", src = src_file, line = line, level = level, msg = args);
            mutex.push(string);
        }
    }

    // Flush the lines and write to a log file
    fn flush(&self) {
        let
    }
}
*/
