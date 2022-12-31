use thiserror::Error;

#[derive(Debug, Error)]
pub enum TextureError {
    #[error("Test")]
    Test,
}
