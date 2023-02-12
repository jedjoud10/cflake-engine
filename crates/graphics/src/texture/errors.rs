use image::ImageError;
use thiserror::Error;
use wgpu::TextureFormat;

#[derive(Error, Debug)]
pub enum TextureInitializationError {
    #[error("The given texture format {0:?} is not supported with the given options")]
    FormatNotSupported(TextureFormat),

    #[error("The number of texels ({0}) does not match up with the number of theoretical texels defined in the dimensions ({1}x{2}x{3})")]
    TexelDimensionsMismatch(usize, u32, u32, u32),

    #[error("Tried creating a mip map for a NPOT texture")]
    MipMapGenerationNPOT,

    #[error("The mip level of {0} does not contain the appropriate number of texels (expected {1}, found {2})")]
    UnexpectedMipLevelTexelCount(u8, u64, u64)
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
    ImageError(ImageError),
}

#[derive(Error, Debug)]
#[error("Cannot create a sampler for texture since it does not have the approperiate usage flags")]
pub struct TextureSamplerError;

#[derive(Error, Debug)]
pub enum TextureAsTargetError {
    #[error("Cannot use the texture mip level as a render target since it does not have the appropriate usage flags")]
    MipLevelMissingFlags,

    #[error("Cannot use the whole texture as a render target since it does not have the appropriate usage flags")]
    WholeTextureMissingFlags,

    #[error("Cannot use the whole texture as a render target since it contains multiple mip levels")]
    WholeTextureMultipleMips
}
