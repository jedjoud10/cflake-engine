use thiserror::Error;

#[derive(Error, Debug)]
#[error("Cannot use the window as a render target since the backing texture was not acquired yet / already been presented")]
pub struct WindowAsTargetError;
