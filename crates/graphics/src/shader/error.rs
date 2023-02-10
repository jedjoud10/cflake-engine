use assets::AssetLoadError;
use naga::{valid::ValidationError, WithSpan};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShaderPreprocessorError {
    #[error("IncludeError: {0}")]
    FileAssetError(AssetLoadError),

    #[error("Snippet {0} was not defined")]
    SnippetNotDefined(String),

    #[error("Constant {0} was not defined")]
    ConstantNotDefined(String),
}

#[derive(Error, Debug)]
pub enum ShaderCompilationError {
    #[error("{0:?}")]
    PreprocessorError(ShaderPreprocessorError),

    #[error("{0:?}")]
    ParserError(Vec<naga::front::glsl::Error>),

    #[error("{0}")]
    SpirvOutError(naga::back::spv::Error),

    #[error("{0}")]
    NagaValidationError(WithSpan<ValidationError>),
}
