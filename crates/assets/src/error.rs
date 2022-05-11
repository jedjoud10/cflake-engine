// An error that occurs when we try to load an asset and we fail
pub enum LoadError<'a> {
    // The file is missing or the path is invalid
    Invalid(&'a str)
}

impl<'a> std::error::Error for LoadError<'a> {}

impl<'a> std::fmt::Debug for LoadError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(path) => write!(f, "Asset not found. Path: {}", path),
        }
    }
}

impl<'a> std::fmt::Display for LoadError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}