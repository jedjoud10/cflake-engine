// An error that occurs when we try to load an asset and we fail
pub enum LoadError {
    // The file is missing or the path is invalid
    Invalid(String),

    // The asset cannot be loaded because the extension is mismatching
    ExtensionMismatch(String),
}

impl std::error::Error for LoadError {}

impl std::fmt::Debug for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadError::Invalid(path) => write!(f, "Asset not found. Path: {}", path),
            LoadError::ExtensionMismatch(extension) => write!(f, "Asset extension mismatched. Current extension: {}", extension),
        }
    }
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
