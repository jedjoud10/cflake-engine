use assets::AssetLoadError;
use thiserror::Error;

use crate::ModuleVisibility;

#[derive(Error, Debug)]
pub enum ShaderError {
    #[error("{0}")]
    Compilation(ShaderCompilationError),

    #[error("{0}")]
    Reflection(ShaderReflectionError),
}

#[derive(Error, Debug)]
pub enum ShaderCompilationError {
    #[error("ShaderC compilation error. Check logs")]
    ShaderC,
}

#[derive(Error, Debug)]
pub enum BufferValidationError {}

#[derive(Error, Debug)]
pub enum TextureValidationError {}

#[derive(Error, Debug)]
pub enum SamplerValidationError {}

#[derive(Error, Debug)]
pub enum PushConstantValidationError {
    #[error("The defined push constant size ({0}) is greater than the adapter's supported push constant size ({1})")]
    PushConstantSizeTooBig(u32, u32),

    #[error("The defined push constant ranges cannot be merged since there is a visibility intersection")]
    PushConstantVisibilityIntersect,
}

#[derive(Error, Debug)]
pub enum ShaderReflectionError {
    #[error("{0}")]
    PushConstantValidation(PushConstantValidationError), 

    #[error("{0}")]
    BufferValidation(BufferValidationError),

    #[error("{0}")]
    TextureValidation(TextureValidationError),

    #[error("{0}")]
    SamplerValidation(SamplerValidationError),

    #[error("The shader defined resource {0} is not defined in the Compiler")]
    NotDefinedInCompiler(String),

    #[error("The Compiler defined resource {0} is not defined in the shader")]
    NotDefinedInShader(String),
}
