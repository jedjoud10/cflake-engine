use std::fmt::Display;

use image::ImageError;
use thiserror::Error;
use wgpu::TextureFormat;

#[derive(Error, Debug)]
pub enum TextureInitializationError {
    #[error("The given texture format {0:?} is not supported with the given options")]
    FormatNotSupported(TextureFormat),

    #[error("The number of texels ({count}) does not match up with the number of theoretical texels defined in the dimensions ({h}x{w}x{d})")]
    TexelDimensionsMismatch {
        count: usize,
        w: u32,
        h: u32,
        d: u32,
    },

    #[error("Tried creating a texture with extent above the physical device's max texture size")]
    ExtentLimit,

    #[error("Tried creating a texture with extent of 0 in any axii")]
    InvalidExtent,

    #[error("Tried creating a mip map for a NPOT texture")]
    MipMapGenerationNPOT,

    #[error("The given texture size is not valid for the block compression algorithm given")]
    SizeNotValidForCompression,

    #[error("The given texture usage contains the READ flag, but there isn't the COPY_SRC flag")]
    ReadableWithoutCopySrc,

    #[error("The given texture usage contains the WRITE flag, but there isn't the COPY_DST flag")]
    WritableWithoutCopyDst,

    #[error("The given texture data is pre-initialized, but there isn't the COPY_DST flag")]
    PreinitializedWithoutCopyDst,

    #[error("The mip level of {level} does not contain the appropriate number of texels (expected {expected}, found {found})")]
    UnexpectedMipLevelTexelCount {
        level: u8,
        expected: u64,
        found: u64,
    },
}

#[derive(Error, Debug)]
pub enum TextureMipLevelError {
    #[error("The given mip level was already mutably borrowed")]
    BorrowedMutably,

    #[error("The given mip level was already immutably borrowed")]
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
pub enum MipLevelReadError {
    #[error("The given source region would overflow the region of the mip-level")]
    InvalidRegion,

    #[error("The mip-level cannot be read since the texture's TextureUsages do not contain READ")]
    NonReadable,
}

#[derive(Error, Debug)]
pub enum MipLevelWriteError {
    #[error("The given source region would overflow the region of the mip-level")]
    InvalidRegion,

    #[error("The mip-level cannot be written since the texture's TextureUsages do not contain WRITE")]
    NonWritable,
}

#[derive(Error, Debug)]
pub enum MipLevelClearError {}

#[derive(Error, Debug)]
pub enum MipLevelCopyError {}

#[derive(Error, Debug)]
pub enum TextureResizeError {
    #[error("Tried resizing a texture which contains mip maps, which isn't supported *yet*")]
    MipMappingUnsupported,

    #[error("Tried resizing a texture above the physical device's max texture size")]
    ExtentLimit,

    #[error("Tried resizing a texture with extent of 0 in any axii")]
    InvalidExtent,

    #[error("Tried resizing a texture, but texture mode isn't TextureMode::Resizable")]
    NotResizable,
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
    WholeTextureMultipleMips,
}
