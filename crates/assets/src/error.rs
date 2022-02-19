use std::fmt;

// Asset metadata load error
#[derive(Debug)]
pub struct AssetLoadError {
    pub file_path: String,
}

impl AssetLoadError {
    pub fn new(path: &str) -> Self {
        Self { file_path: path.to_string() }
    }
}

impl fmt::Display for AssetLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Could not load asset file '{}'!", self.file_path)
    }
}

impl std::error::Error for AssetLoadError {
    fn description(&self) -> &str {
        &self.file_path
    }
}
