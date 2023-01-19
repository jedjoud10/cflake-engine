use thiserror::Error;

use crate::VertexAttribute;

#[derive(Error, Debug)]
pub enum PipelineInitializationError {
    #[error("{0}")]
    VertexConfigError(PipelineVertexConfigError)
}

#[derive(Error, Debug)]
pub enum PipelineVertexConfigError {
    /*
    #[error("The given vertex configuration is empty")]
    Empty,

    #[error("The binding {0} is defined multiple times")]
    BindingRedefined(u32),

    #[error("The vertex attribute a index {0} is referencing binding {1}, which is not present")]
    AttributeBindsOutOfBounds(u32, u32),

    #[error("The vertex binding {0} is not being used by any vertex attributes")]
    BindingNotUsed(u32),
    */
}