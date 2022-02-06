use std::fmt;

// An error related to the global components
#[derive(Debug)]
pub struct GlobalError {
    details: String,
}

impl GlobalError {
    pub fn new(msg: String) -> Self {
        Self { details: msg }
    }
}

impl fmt::Display for GlobalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for GlobalError {
    fn description(&self) -> &str {
        &self.details
    }
}