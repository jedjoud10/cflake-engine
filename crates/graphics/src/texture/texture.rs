use std::{
    cell::Cell,
    marker::PhantomData,
    mem::transmute,
    num::{NonZeroU32, NonZeroU8},
    ops::{Bound, RangeBounds},
    sync::Arc,
};

use itertools::Itertools;
use smallvec::SmallVec;
use wgpu::{SamplerDescriptor, TextureDescriptor, TextureViewDescriptor};

use super::{
    Extent, LayeredOrigin, Origin, Region, Sampler,
    SamplerSettings, SamplerWrap, TextureInitializationError,
    TextureMipLevelError, TextureMipMaps, TextureSamplerError, TextureUsage, TextureViewDimension,
    TextureViewMut, TextureViewRef, TextureViewSettings, ViewAsTargetError,
};
use crate::pod::GpuPod;
use crate::context::Graphics;
use crate::format::{Texel, TexelSize};
use crate::pass::RenderTarget;

// A texture is an abstraction over Vulkan images to allow us to access/modify them with ease
// A texture is a container of multiple texels (like pixels, but for textures) that are stored on the GPU
// This trait is implemented for all variants of textures (1D, 2D, 3D, Layered2D, CubeMap)
pub trait Texture: Sized + 'static {
    // Texel region (position + extent)
    type Region: Region;

    // Texel layout that we will use internally
    type T: Texel;

    // Create a new texture with some possibly predefined data
    // Make it optional, and do runtime checks to check if the texture usage is TextureUsage::SAMPLED,
    // and enforce the sampling parameter if it is
    fn from_texels(
        graphics: &Graphics,
        texels: Option<&[<Self::T as Texel>::Storage]>,
        extent: <Self::Region as Region>::E,
        usage: TextureUsage,
        views: &[TextureViewSettings],
        mipmaps: TextureMipMaps<Self::T>,
    ) -> Result<Self, TextureInitializationError> {
        let format = <Self::T as Texel>::format();

        // Make sure the number of texels matches up with the dimensions
        if let Some(texels) = texels {
            if <Self::Region as Region>::volume(extent) as usize != texels.len() {
                return Err(TextureInitializationError::TexelDimensionsMismatch {
                    count: texels.len(),
                    w: extent.width(),
                    h: extent.height(),
                    d: extent.layers(),
                });
            }
        }

        // Get the image type using the dimensionality
        let dimension = <Self::Region as Region>::dimension();
        let extent_3d = extent_to_extent3d::<Self::Region>(extent);

        // If the extent contains a 0 in any axii, it's invalid
        if !extent.is_valid() {
            return Err(TextureInitializationError::InvalidExtent);
        }

        // If the extent is greater than the physical limits, it's invalid
        if !size_within_limits::<Self::Region>(&graphics, extent) {
            return Err(TextureInitializationError::ExtentLimit);
        }

        // Return an error if the texture usage flags are invalid
        if usage.contains(TextureUsage::READ) && !usage.contains(TextureUsage::COPY_SRC) {
            return Err(TextureInitializationError::ReadableWithoutCopySrc);
        } else if usage.contains(TextureUsage::WRITE) && !usage.contains(TextureUsage::COPY_DST) {
            return Err(TextureInitializationError::WritableWithoutCopyDst);
        } else if !usage.contains(TextureUsage::COPY_DST)
            && (texels.is_some()
                || match mipmaps {
                    TextureMipMaps::Manual { mips } => mips.len() > 0,
                    _ => false,
                })
        {
            return Err(TextureInitializationError::PreinitializedWithoutCopyDst);
        }

        // Get optimal texture usage
        let usages = texture_usages(usage);

        // Check if the format is valid for the given usage flag
        let texture_format_features = graphics.adapter().get_texture_format_features(format);
        if !texture_format_features.allowed_usages.contains(usages) {
            return Err(TextureInitializationError::FormatNotSupported(format));
        }

        // Get the number of mip levels for this texture
        let levels = mip_levels::<Self::T, Self::Region>(&mipmaps, extent)?;

        // Config for the Wgpu texture
        let descriptor = TextureDescriptor {
            size: extent_3d,
            mip_level_count: levels,
            sample_count: 1,
            dimension,
            format,
            usage: usages,
            label: None,
            view_formats: &[],
        };

        // Create the raw WGPU texture
        let texture = graphics.device().create_texture(&descriptor);
        log::debug!(
            "Creating texture, {dimension:?}, <{}>, {}x{}x{}",
            std::any::type_name::<Self::T>(),
            extent.width(),
            extent.height(),
            extent.layers(),
        );

        // Create a "zeroed" origin
        let origin = <<Self::Region as Region>::O as Default>::default();

        // Always write to the first mip level
        if let Some(texels) = texels {
            write_to_level::<Self::T, Self::Region>(origin, extent, texels, &texture, 0, graphics);
        }

        // Fill the texture mips with the appropriate data
        match mipmaps {
            TextureMipMaps::Manual { mips } => {
                log::debug!(
                    "Creating manual mip-map layers for texture (max = {}) with extent {}x{}x{}",
                    mips.len(),
                    extent.width(),
                    extent.height(),
                    extent.layers()
                );

                // Manual mip map generation
                let iter = mips
                    .iter()
                    .take(levels as usize - 1)
                    .enumerate()
                    .map(|(x, y)| (x + 1, y));
                for (i, texels) in iter {
                    // Downscale the texture extent by two
                    let downscaled_extent = extent.mip_level_dimensions(i as u32);

                    // Write bytes to the level
                    write_to_level::<Self::T, Self::Region>(
                        origin,
                        downscaled_extent,
                        texels,
                        &texture,
                        i as u32,
                        graphics,
                    );
                }
            }
            _ => {}
        }

        // TODO: Force the user to set the first view as a whole texture view if it is a SAMPLED or TARGET texture

        // Create the texture views that the user set up
        let layers = <Self::Region as Region>::layers(extent);
        let views = create_texture_views::<Self::T, Self::Region>(&texture, format, extent, views)?;

        Ok(unsafe {
            Self::from_raw_parts(graphics, texture, views, extent, usage)
        })
    }

    // Get the texture's region (origin state is default)
    fn region(&self) -> Self::Region {
        Self::Region::from_extent(self.dimensions())
    }

    // Checks if we can access a region of the texture
    fn is_region_accessible(&self, region: Self::Region) -> bool {
        self.region().is_larger_than(region)
    }

    // Get the texture's dimensions
    fn dimensions(&self) -> <Self::Region as Region>::E;

    // Get the texture's usage
    fn usage(&self) -> TextureUsage;

    // Get the underlying WGPU Texture immutably
    fn raw(&self) -> &wgpu::Texture;

    // Get the underlying WGPU views immutably
    fn raw_views(&self) -> &[(wgpu::TextureView, TextureViewSettings)];

    // Get the number of mip levels currently stored within the texture
    fn levels(&self) -> u32 {
        self.raw().mip_level_count()
    }

    // Get the number of layers (3D depth or layers) currently stored within the texture
    fn layers(&self) -> u32 {
        <Self::Region as Region>::layers(self.dimensions())
    }

    // Get a single immutable view of the texture
    fn view(&self, index: usize) -> Option<TextureViewRef<Self>> {
        self.raw_views()
            .get(index)
            .map(|(view, settings)| TextureViewRef {
                texture: self,
                view,
                settings,
            })
    }

    // Get a single mutable view of the texture
    fn view_mut(&mut self, index: usize) -> Option<TextureViewMut<Self>> {
        self.raw_views()
            .get(index)
            .map(|(view, settings)| TextureViewMut {
                texture: self,
                view,
                settings,
            })
    }

    // Try to use the whole texture as a renderable target. This will fail if the texture isn't supported as render target
    // or if it's dimensions don't correspond to a 2D image
    fn as_render_target(&mut self) -> Result<RenderTarget<Self::T>, ViewAsTargetError> {
        if !self.usage().contains(TextureUsage::TARGET) {
            return Err(ViewAsTargetError::MissingTargetUsage);
        }

        if self.levels() > 1 {
            return Err(ViewAsTargetError::ViewMultipleMips);
        }

        if self.region().extent().layers() > 1 {
            return Err(ViewAsTargetError::RegionIsNot2D);
        }

        Ok(RenderTarget {
            _phantom: PhantomData,
            view: self.view(0).unwrap().view,
        })
    }

    // Get the stored graphics context
    fn graphics(&self) -> Graphics;

    // Create a texture struct from it's raw components
    // This will simply create the texture struct, and it assumes
    // that the texture was already created externally with the right parameters
    unsafe fn from_raw_parts(
        graphics: &Graphics,
        texture: wgpu::Texture,
        views: Vec<(wgpu::TextureView, TextureViewSettings)>,
        dimensions: <Self::Region as Region>::E,
        usage: TextureUsage,
    ) -> Self;
}

// Get the number of mip levels that the texture should use
pub(crate) fn mip_levels<T: Texel, R: Region>(
    mipmaps: &TextureMipMaps<T>,
    extent: R::E,
) -> Result<u32, TextureInitializationError> {
    let max_mip_levels = if matches!(mipmaps, TextureMipMaps::Disabled) {
        1u8 as u32
    } else {
        let max = R::levels(extent).ok_or(TextureInitializationError::MipMapGenerationNPOT)?;

        // If we are using compression, we must make sure the lowest level is at least the block size
        match T::size() {
            TexelSize::Uncompressed(_) => max.get() as u32,
            TexelSize::Compressed(c) => {
                let val = max.get() as u32;
                let logged = (c.block_size() as f32).log2() + 2.0;
                val - (logged as u32)
            }
        }
    };

    // Convert Auto to Zeroed (since if this texture was loaded from disk, it would've been Manual instead)
    let levels = match *mipmaps {
        // Automatic mip level generation, but fills mips with zeroes
        TextureMipMaps::Zeroed { clamp } => {
            let max = clamp.map(|x| x.get()).unwrap_or(u8::MAX);
            (max as u32).min(max_mip_levels)
        }

        // Manual mip level generatation, but fills mips with corresponding data
        TextureMipMaps::Manual { mips } => {
            let levels = mips.len() as u32 + 1;
            levels.min(max_mip_levels)
        }

        // No mip maps
        TextureMipMaps::Disabled => 1,
    };

    log::debug!(
        "Calculated {levels} mip levels for texture with extent {}x{}x{}",
        extent.width(),
        extent.height(),
        extent.layers(),
    );

    Ok(levels)
}

// Creates the textures views for the base texture and it's corresponding mips
// The texture mip/layer ranges for each view are specified in the TextureView struct
// This also handles validation for us automatically
fn create_texture_views<T: Texel, R: Region>(
    texture: &wgpu::Texture,
    format: wgpu::TextureFormat,
    extent: R::E,
    views: &[TextureViewSettings],
) -> Result<Vec<(wgpu::TextureView, TextureViewSettings)>, TextureInitializationError> {
    log::debug!(
        "Creating level views for texture with extent {}x{}x{}",
        extent.width(),
        extent.height(),
        extent.layers(),
    );

    let views = views.into_iter().unique();
    let aspect = texture_aspect::<T>();
    Ok(views
        .map(|setting| {
            (
                texture.create_view(&wgpu::TextureViewDescriptor {
                    label: None,
                    format: Some(format),
                    dimension: Some(setting.dimension),
                    aspect,
                    base_mip_level: setting.base_mip_level,
                    mip_level_count: setting.mip_level_count,
                    base_array_layer: setting.base_array_layer,
                    array_layer_count: setting.array_layer_count,
                }),
                *setting,
            )
        })
        .collect::<Vec<_>>())
}

// Create an image data layout based on the extent and texel type
// This should support compression textures too, though I haven't tested all cases yet
pub(crate) fn create_image_data_layout<T: Texel, R: Region>(extent: R::E) -> wgpu::ImageDataLayout {
    let size = T::size();

    // Bytes per row change if we are using compressed textures
    let bytes_per_row = match size {
        TexelSize::Uncompressed(size) => Some(size * extent.width()),
        TexelSize::Compressed(compression) => {
            // TODO: Actually try understanding wtf bytes_per_row means when using compression
            Some(compression.bytes_per_block() * (extent.width() / compression.block_size()))
        }
    };

    wgpu::ImageDataLayout {
        offset: 0,
        bytes_per_row,
        rows_per_image: match (R::dimension(), R::view_dimension()) {
            (wgpu::TextureDimension::D3, wgpu::TextureViewDimension::D3) => Some(extent.width()),

            (wgpu::TextureDimension::D2, wgpu::TextureViewDimension::Cube) => Some(extent.width()),

            (wgpu::TextureDimension::D2, wgpu::TextureViewDimension::D2Array) => {
                Some(extent.width())
            }

            _ => None,
        },
    }
}

// Write texels to a single level of a texture
// Assumes that the origin and extent are in mip-space
pub(crate) fn write_to_level<T: Texel, R: Region>(
    origin: R::O,
    extent: R::E,
    texels: &[T::Storage],
    texture: &wgpu::Texture,
    level: u32,
    graphics: &Graphics,
) {
    // This should handle compression types too
    let image_data_layout = create_image_data_layout::<T, R>(extent);

    // Convert the texels to bytes
    let bytes = bytemuck::cast_slice::<T::Storage, u8>(texels);

    // Create the image copy texture descriptor
    let image_copy_texture = wgpu::ImageCopyTexture {
        texture: texture,
        mip_level: level,
        origin: origin_to_origin3d(origin),
        aspect: texture_aspect::<T>(),
    };

    // Write to the mip level of the texture
    graphics.0.queue.write_texture(
        image_copy_texture,
        bytes,
        image_data_layout,
        extent_to_extent3d::<R>(extent)
    );
}

// Read texels from a single level of a texture
// Assumes that the origin and extent are in mip-space
pub(crate) fn read_from_level<T: Texel, R: Region>(
    origin: R::O,
    extent: R::E,
    texels: &mut [T::Storage],
    texture: &wgpu::Texture,
    level: u32,
    graphics: &Graphics,
) {
    todo!()
}

// Copy a sub-region from another level into this level
// The texture can have different regions, but the same type
pub fn copy_subregion_from<'a, T: Texture, O: Texture<T = T::T>>(
    write: impl Into<TextureViewMut<'a, T>>,
    read: impl Into<TextureViewRef<'a, O>>,
    src_subregion: Option<O::Region>,
    dst_subregion: Option<T::Region>,
    graphics: &Graphics,
) {
    let write = write.into();
    let read = read.into();

    //let mut encoder = graphics.acquire();
}

// Check if the given extent is valid within device limits
fn size_within_limits<R: Region>(graphics: &Graphics, extent: R::E) -> bool {
    // Create the max possible texture size from device limits
    let limits = graphics.device().limits();
    let max = match R::dimension() {
        wgpu::TextureDimension::D1 => limits.max_texture_dimension_1d,
        wgpu::TextureDimension::D2 => limits.max_texture_dimension_2d,
        wgpu::TextureDimension::D3 => limits.max_texture_dimension_3d,
    };
    let max = <R::E as Extent>::broadcast(max);

    // Check if the new texture size is physically possible
    max.is_larger_than(extent)
}

// Convert the given origin to an Origin
pub(crate) fn origin_to_origin3d<O: Origin>(origin: O) -> wgpu::Origin3d {
    wgpu::Origin3d {
        x: origin.x(),
        y: origin.y(),
        z: origin.z(),
    }
}

// Convert the given extent to an Extent3D
pub(crate) fn extent_to_extent3d<R: Region>(dimensions: R::E) -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: dimensions.width(),
        height: dimensions.height(),
        depth_or_array_layers: R::layers(dimensions),
    }
}

// Get the texture usages from the texture usage wrapper
// Does not check for validity
pub(crate) fn texture_usages(usage: TextureUsage) -> wgpu::TextureUsages {
    let mut usages = wgpu::TextureUsages::empty();

    if usage.contains(TextureUsage::SAMPLED) {
        usages |= wgpu::TextureUsages::TEXTURE_BINDING;
    }

    if usage.contains(TextureUsage::TARGET) {
        usages |= wgpu::TextureUsages::RENDER_ATTACHMENT;
    }

    if usage.contains(TextureUsage::STORAGE) {
        usages |= wgpu::TextureUsages::STORAGE_BINDING;
    }

    if usage.contains(TextureUsage::COPY_SRC) {
        usages |= wgpu::TextureUsages::COPY_SRC;
    }

    if usage.contains(TextureUsage::COPY_DST) {
        usages |= wgpu::TextureUsages::COPY_DST;
    }

    usages
}

// Get the texture aspect based on the texel type
pub(crate) fn texture_aspect<T: Texel>() -> wgpu::TextureAspect {
    match T::channels() {
        crate::format::TexelChannels::Depth => wgpu::TextureAspect::DepthOnly,
        crate::format::TexelChannels::Stencil => wgpu::TextureAspect::StencilOnly,
        _ => wgpu::TextureAspect::All,
    }
}
