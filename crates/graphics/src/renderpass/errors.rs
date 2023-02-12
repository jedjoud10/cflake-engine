use thiserror::Error;

#[derive(Error, Debug)]
pub enum RenderPassInitializationError {}

#[derive(Error, Debug)]
#[error("t")]
pub struct RenderPassBeginError;
