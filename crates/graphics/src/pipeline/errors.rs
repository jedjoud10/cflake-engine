use thiserror::Error;

#[derive(Error, Debug)]
pub enum PipelineInitializationError {
    #[error("{0}")]
    VertexConfigError(PipelineVertexConfigError),
}

#[derive(Error, Debug)]
pub enum PipelineVertexConfigError {
    #[error("The shader location {0} is used multiple times")]
    ShaderLocationRedefined(u32),
}
