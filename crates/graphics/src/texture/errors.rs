use thiserror::Error;
use vulkan::vk;

#[derive(Error, Debug)]
pub enum TextureInitializationError {
    #[error("The given texture format {0:?} is not supported on the physical device")]
    FormatNotSupported(vk::Format),
}

// Texture error that is returned from each texture command
#[derive(Error, Debug)]
pub enum TextureError {
    #[error("{0}")]
    Initialization(TextureInitializationError)
}
