use thiserror::Error;

#[derive(Error, Debug)]
pub enum PipelineInitializationError {
    #[error("{0}")]
    VertexConfigError(PipelineVertexConfigError),

    #[error("Stencil layout is enabled, although stencil configuration is missing")]
    MissingStencilConfig,
    
    #[error("Depth layout is enabled, although depth configuration is missing")]
    MissingDepthConfig,
}

#[derive(Error, Debug)]
pub enum PipelineVertexConfigError {
    #[error("The shader location {0} is used multiple times")]
    ShaderLocationRedefined(u32),
}