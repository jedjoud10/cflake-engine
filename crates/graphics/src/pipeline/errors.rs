use thiserror::Error;

#[derive(Error, Debug)]
pub enum PipelineInitializationError {
    #[error("{0}")]
    VertexConfigError(PipelineVertexConfigError),

    #[error("Stencil layout is enabled, although stencil configuration is missing")]
    MissingStencilConfig,
    
    #[error("Depth layout is enabled, although depth configuration is missing")]
    MissingDepthConfig,

    #[error("{0}")]
    InvalidBindings(PipelineBindingsError),
}

#[derive(Error, Debug)]
pub enum PipelineVertexConfigError {
    #[error("The shader location {0} is used multiple times")]
    ShaderLocationRedefined(u32),
}

#[derive(Error, Debug)]
pub enum PipelineBindingsError {
}