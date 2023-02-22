use assets::AssetLoadError;
use naga::{valid::ValidationError, WithSpan};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShaderPreprocessorError {
    #[error("Shader include directive is invalid or incomplete")]
    InvalidIncludeDirective,

    #[error("Shader include cylcic reference detected")]
    IncludeCyclicReference,

    #[error("IncludeError: {0}")]
    FileAssetError(AssetLoadError),

    #[error("Snippet {0} was not defined")]
    SnippetNotDefined(String),
}

#[derive(Error, Debug)]
pub enum ShaderCompilationError {
    #[error("{0:?}")]
    PreprocessorError(ShaderPreprocessorError),

    #[error("Shader validation error. Check logs")]
    ValidationError,
}
