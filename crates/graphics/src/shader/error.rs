use assets::AssetLoadError;
use naga::{valid::ValidationError, WithSpan};
use thiserror::Error;


// Only used internally to make error handling a bit easier
// This will be converted to a string eventually
#[derive(Error, Debug)]
pub(crate) enum ShaderIncludeError {
    #[error("Shader include cylcic reference detected")]
    IncludeCyclicReference,

    #[error("IncludeError: {0}")]
    FileAssetError(AssetLoadError),

    #[error("Snippet {0} was not defined")]
    SnippetNotDefined(String),
}

#[derive(Error, Debug)]
#[error("ShaderC compilation error. Check logs")]
pub struct ShaderCompilationError;
