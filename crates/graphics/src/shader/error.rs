use assets::AssetLoadError;
use naga::{valid::ValidationError, WithSpan};
use thiserror::Error;


#[derive(Error, Debug)]
pub enum ShaderCompilationError {    
    #[error("ShaderC compilation error. Check logs")]
    ShaderC,
}
