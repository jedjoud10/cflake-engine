use std::error::Error;
use thiserror::Error;

// Error that occurs when we try to load an asset
#[derive(Error, Debug)]
pub enum AssetLoadError {
    #[error("Invalid '{0}' extension in file path")]
    InvalidExtension(String),

    #[error("Cannot find file at path '{0}'")]
    DynamicNotFound(String),

    #[error("Could not convert to OS str")]
    InvalidOsStr,

    #[error("Missing extension in file path")]
    MissingExtension,

    #[error("User asset path was not specified")]
    UserPathNotSpecified,

    #[error("Deserialization error {0}")]
    BoxedDeserialization(Box<dyn Error + Send + Sync>),
}
