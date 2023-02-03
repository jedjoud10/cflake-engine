use assets::AssetLoadError;
use naga::{WithSpan, valid::ValidationError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShaderIncludeError {
    #[error("{0}")]
    FileAssetError(AssetLoadError),

    #[error("A")]
    SnippetNotDefined,
}

#[derive(Error, Debug)]
pub enum ShaderCompilationError {
    #[error("A")]
    MissingConst,

    #[error("{0:?}")]
    ParserError(Vec<naga::front::glsl::Error>),

    #[error("{0}")]
    SpirvOutError(naga::back::spv::Error),

    #[error("{0}")]
    NagaValidationError(WithSpan<ValidationError>),
    
    #[error("{0}")]
    ReflectionError(String),
}
