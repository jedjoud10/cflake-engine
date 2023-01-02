use thiserror::Error;
use vulkan::vk;

#[derive(Error, Debug)]
pub enum TextureInitializationError {
    #[error("The given texture format {0:?} is not supported on the physical device")]
    FormatNotSupported(vk::Format),

    #[error("The number of texels ({0}) does not match up with the number of theoretical texels defined in the dimensions ({1}x{2}x{3})")]
    TexelDimensionsMismatch(usize, u32, u32, u32)
}

#[derive(Error, Debug)]
pub enum TextureMipLayerError {
    #[error("The given mip layer was already mutably borrowed")]
    BorrowedMutably,
    
    #[error("The given mip layer was already immutably borrowed")]
    BorrowedImmutably,
}

#[derive(Error, Debug)]
pub enum TextureAssetLoadError {
    #[error("{0}")]
    Initialization(TextureInitializationError),
    
    #[error("{0}")]
    Deserialization(image::ImageError),
}

#[derive(Error, Debug)]
#[error("Cannot create a sampler for texture since it does not have the approperiate usage flags")]
pub struct TextureSamplerError;