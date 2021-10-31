use std::fmt;

// Asset metadata load error
#[derive(Debug)]
pub struct AssetMetadataLoadError {
    details: String,
}

impl AssetMetadataLoadError {
    pub fn new(msg: String) -> Self {
        Self { details: msg }
    }
    pub fn new_str(msg: &str) -> Self {
        Self { details: msg.to_string() }
    }
}

impl fmt::Display for AssetMetadataLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for AssetMetadataLoadError {
    fn description(&self) -> &str {
        &self.details
    }
}

// Object load error
#[derive(Debug)]
pub struct ObjectLoadError {
    details: String,
}

impl ObjectLoadError {
    pub fn new(msg: String) -> Self {
        Self { details: msg }
    }
    pub fn new_str(msg: &str) -> Self {
        Self { details: msg.to_string() }
    }
}

impl fmt::Display for ObjectLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for ObjectLoadError {
    fn description(&self) -> &str {
        &self.details
    }
}
