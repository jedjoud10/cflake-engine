use crate::SaverLoader;
use std::time::SystemTime;

// Logs errors catched by the error catcher
#[derive(Clone)]
pub struct ErrorLogger {
    // The currently saved errors that we must log
    pub errors: Vec<String>,
}

impl ErrorLogger {
    // Save an error so we can log it later
    pub fn catch_error(&mut self, error: String) {
        // Log the timestamp as well
        let t = std::time::SystemTime::now();
        let t = t.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        // Format the string
        let error_string = format!("S: {}, E: {}", t, error);
        self.errors.push(error_string);
    }
    // Actually log the errors to the log file
    pub fn log_errors(&self, saver_loader: &mut SaverLoader) {
        //saver_loader.save_string("", string)
    }
}
