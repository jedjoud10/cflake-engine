use std::error::Error;
use thiserror::Error;

/// Error that occurs when we try to load an asset
#[derive(Error, Debug)]
pub enum AssetLoadError {
    /// Invalid extension when parsing file path
    #[error("Invalid '{0}' extension in file path")]
    InvalidExtension(String),

    /// Dynamic file does not exist
    #[error("Cannot load dynamic file at path '{0}'")]
    DynamicNotFound(String),

    /// Cached file does not exist
    #[error("Cannot load cached filed bytes at path '{0}'")]
    CachedNotFound(String),

    /// Cannot convert to OS Str
    #[error("Could not convert to OS str")]
    InvalidOsStr,

    /// Missing extension in file path
    #[error("Missing extension in file path")]
    MissingExtension,

    /// User path not specified (Dynamic loading)
    #[error("User asset path was not specified")]
    UserPathNotSpecified,

    /// Error when deserializing asset
    #[error("Deserialization error {0}")]
    BoxedDeserialization(Box<dyn Error + Send + Sync>),
}
