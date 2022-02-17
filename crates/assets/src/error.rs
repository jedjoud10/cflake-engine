use std::fmt;

// Asset metadata load error
#[derive(Debug)]
pub struct AssetLoadError {
    pub details: String,
}

impl AssetLoadError {
    pub fn new(msg: String) -> Self {
        Self { details: msg }
    }
    pub fn new_str(msg: &str) -> Self {
        Self { details: msg.to_string() }
    }
}

impl fmt::Display for AssetLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for AssetLoadError {
    fn description(&self) -> &str {
        &self.details
    }
}
