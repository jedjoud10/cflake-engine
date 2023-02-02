use assets::AssetLoadError;
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

    #[error("{0}")]
    TranslationError(shaderc::Error),

    #[error("{0}")]
    ReflectionError(String),
}
