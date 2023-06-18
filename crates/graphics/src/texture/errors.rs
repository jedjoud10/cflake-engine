use std::fmt::Display;

use image::ImageError;
use thiserror::Error;
use wgpu::TextureFormat;

use crate::RawTexelsError;

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

    #[error("The texture is set with a usage of SAMPLED, but there are no sampling settings defined (set to none)")]
    TextureUsageSampledMissingSettings,

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

    #[error("The given mip level ({0}) is out of the mip levels within the texture ({1})")]
    OutOfRange(u8, u8),
}

#[derive(Error, Debug)]
pub enum TextureAssetLoadError {
    #[error("{0}")]
    Initialization(TextureInitializationError),

    #[error("{0}")]
    RawTexelsError(RawTexelsError),
}

#[derive(Error, Debug)]
pub enum ViewReadError {
    #[error("The given source region would overflow the region of the view")]
    InvalidRegion,

    #[error("The view cannot be read since the texture's TextureUsages do not contain READ")]
    NonReadable,

    #[error("Destination data length does not match up with region")]
    DstLenMismatch
}

#[derive(Error, Debug)]
pub enum ViewWriteError {
    #[error("The given source region would overflow the region of the view")]
    InvalidRegion,

    #[error(
        "The view cannot be written since the texture's TextureUsages do not contain WRITE"
    )]
    NonWritable,

    #[error("Source data length does not match up with region")]
    SrcLenMismatch,

    #[error("Cannot write to multiple mip levels at once")]
    MultipleMipLevels,
}

#[derive(Error, Debug)]
pub enum ViewClearError {
    #[error("The given source region would overflow the region of the view")]
    InvalidRegion,

    #[error(
        "The view cannot be cleared since the texture's TextureUsages do not contain WRITE"
    )]
    NonWritable,
}

#[derive(Error, Debug)]
pub enum ViewCopyError {
    #[error("The given source region would overflow the region of the view")]
    InvalidSrcRegion,

    #[error("The given destination region would overflow the region of the view")]
    InvalidDstRegion,

    #[error("The view cannot be copied into since the texture's TextureUsages do not contain COPY_DST")]
    NonCopyDst,

    #[error("The view cannot be copied from since the texture's TextureUsages do not contain COPY_SRC")]
    NonCopySrc,

    #[error("The subregions must have the same number of texels to be able to copy them")]
    TexelCountNotEqual,

    #[error("The given texture level cannot be copied if it is a multisampled or depth texture")]
    CannotUseSubregion,

    #[error("Todo")]
    IncompatibleMultiLayerTextures,
}

#[derive(Error, Debug)]
#[error("Cannot create a sampler for texture since it does not have the approperiate usage flags")]
pub struct TextureSamplerError;

#[derive(Error, Debug)]
pub enum ViewAsTargetError {
    #[error("The given source region would overflow the region of the view")]
    InvalidRegion,

    #[error("Cannot use the texture as a render target since it does not have the appropriate usage flags")]
    MissingTargetUsage,

    #[error(
        "Cannot use the texture as a render target because the texture region is layered / 3D"
    )]
    RegionIsNot2D,

    #[error(
        "Cannot use the whole texture as a render target since it contains multiple mip levels"
    )]
    ViewMultipleMips,
}