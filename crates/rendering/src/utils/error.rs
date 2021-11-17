use core::fmt;

// Rendering error
#[derive(Debug, Clone)]
pub struct RenderingError {
    details: String,
}

impl RenderingError {
    pub fn new(msg: String) -> Self {
        Self { details: msg }
    }
    pub fn new_str(msg: &str) -> Self {
        Self { details: msg.to_string() }
    }
}

impl fmt::Display for RenderingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for RenderingError {
    fn description(&self) -> &str {
        &self.details
    }
}
