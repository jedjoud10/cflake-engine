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
pub enum BufferValidationError {
    #[error("The compiler resource is not a uniform buffer, as defined in the shader")]
    NotUniformBuffer,

    #[error("The compiler resource is not a storage buffer, as defined in the shader")]
    NotStorageBuffer,

    #[error("The compiler defined type (of size {compiler}), does not match up with the one defined in the shader (size {shader})")]
    MismatchSize { compiler: usize, shader: usize },

    #[error("The compiler defined buffer storage access {compiler:?} does not match up with the shader defined access {shader:?}")]
    MismatchAccess {
        compiler: spirq::AccessType,
        shader: spirq::AccessType,
    },
}

#[derive(Error, Debug)]
pub enum TextureValidationError {
    #[error("The compiler resource is not a sampled texture, as defined in the shader")]
    NotSampledTexture,

    #[error("The compiler resource is not a storage texture, as defined in the shader")]
    NotStorageTexture,

    #[error("The compiler defined texture format {compiler:?} does not match up with the shader defined format {shader:?}")]
    MismatchFormat {
        compiler: wgpu::TextureFormat,
        shader: wgpu::TextureFormat,
    },

    #[error("The compiler defined view dimensions {compiler:?} does not match up with the shader defined view dimensions {shader:?}")]
    MismatchViewDimension {
        compiler: wgpu::TextureViewDimension,
        shader: wgpu::TextureViewDimension,
    },

    #[error("The compiler defined texture storage access {compiler:?} does not match up with the shader defined access {shader:?}")]
    MismatchAccess {
        compiler: spirq::AccessType,
        shader: spirq::AccessType,
    },

    #[error("The compielr defined texture sample type {compiler:?} does not match up wiuth the shader defined sample type {shader:?}")]
    MismatchSampleType {
        compiler: wgpu::TextureSampleType,
        shader: wgpu::TextureSampleType,
    },
}

#[derive(Error, Debug)]
pub enum SamplerValidationError {
    #[error("The given resource is not a sampler, as defined in the shader")]
    NotSampler,
}

#[derive(Error, Debug)]
pub enum PushConstantValidationError {
    #[error("The defined push constant size ({0}) is greater than the adapter's supported push constant size ({1})")]
    PushConstantSizeTooBig(u32, u32),

    #[error("The defined push constant ranges cannot be merged since there is a visibility intersection")]
    PushConstantVisibilityIntersect,

    #[error("The defined compiler push constant are not defined in the shader or they have a different size requirement")]
    PushConstantNotDefinedOrDiffSized,
}

#[derive(Error, Debug)]
pub enum ShaderReflectionError {
    #[error("{0}")]
    PushConstantValidation(PushConstantValidationError),

    #[error("buffer resource: {resource}, error: {error}")]
    BufferValidation {
        resource: String,
        error: BufferValidationError,
    },

    #[error("texture resource: {resource}, error: {error}")]
    TextureValidation {
        resource: String,
        error: TextureValidationError,
    },

    #[error("sampler resource: {resource}, error: {error}")]
    SamplerValidation {
        resource: String,
        error: SamplerValidationError,
    },

    #[error("The shader defined resource {0} is not defined in the Compiler")]
    NotDefinedInCompiler(String),

    #[error("The compute shader's local workgroup total size is {shader}, whilst the device limit is {limit}")]
    ComputeShaderLocalWorkgroupSizeLimit { shader: u32, limit: u32 },
}
