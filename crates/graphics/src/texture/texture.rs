use std::{
    cell::Cell,
    marker::PhantomData,
    mem::transmute,
    num::{NonZeroU32, NonZeroU8},
    sync::Arc,
};

use smallvec::SmallVec;
use wgpu::{SamplerDescriptor, TextureDescriptor, TextureViewDescriptor};

use crate::{
    Extent, GpuPod, Graphics, LayeredOrigin, MipLevelMut, MipLevelRef, MipLevelsMut, MipLevelsRef,
    Origin, Region, RenderTarget, Sampler, SamplerSettings, SamplerWrap, Texel, TexelSize,
    TextureAsTargetError, TextureInitializationError, TextureMipLevelError, TextureMipMaps,
    TextureMode, TextureResizeError, TextureSamplerError, TextureUsage, ViewDimension,
};

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
        mode: TextureMode,
        usage: TextureUsage,
        sampling: Option<SamplerSettings>,
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
                    d: extent.depth_or_layers(),
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
        let name = utils::pretty_type_name::<Self::T>();
        log::debug!(
            "Creating texture, {dimension:?}, <{name}>, {}x{}x{}",
            extent.width(),
            extent.height(),
            extent.depth_or_layers(),
        );

        // Fetch a new sampler for the given sampling settings (if needed)
        let sampler = if usage.contains(TextureUsage::SAMPLED) {
            let sampling =
                sampling.ok_or(TextureInitializationError::TextureUsageSampledMissingSettings)?;
            Some(crate::get_or_insert_sampler(graphics, sampling))
        } else {
            None
        };

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
                    extent.depth_or_layers()
                );

                // Manual mip map generation
                let iter = mips
                    .iter()
                    .take(levels as usize - 1)
                    .enumerate()
                    .map(|(x, y)| (x + 1, y));
                for (i, texels) in iter {
                    // Downscale the texture extent by two
                    let downscaled_extent = extent.mip_level_dimensions(i as u8);

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

        // Check if we need texture views
        let needs_views = usage.contains(TextureUsage::SAMPLED)
            | usage.contains(TextureUsage::STORAGE)
            | usage.contains(TextureUsage::TARGET);

        // Create the texture views when deemed necessary
        let views = needs_views.then(|| {
            let layers = <Self::Region as Region>::layers(extent);

            create_texture_views::<Self::T, Self::Region>(&texture, format, extent, levels, layers)
        });

        Ok(unsafe {
            Self::from_raw_parts(
                graphics, texture, views, sampler, sampling, extent, usage, mode,
            )
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

    // Get the texture's mode
    fn mode(&self) -> TextureMode;

    // Get the texture's usage
    fn usage(&self) -> TextureUsage;

    // Get the underlying WGPU Texture immutably
    fn raw(&self) -> &wgpu::Texture;

    // Get the sampler associated with this texture
    // Returns none if the texture cannot be sampled
    fn sampler(&self) -> Option<Sampler<Self::T>>;

    // Get the number of mip levels currently stored within the texture
    fn levels(&self) -> u32 {
        self.raw().mip_level_count()
    }

    // Get the number of layers currently stored within the texture
    fn layers(&self) -> u32 {
        <Self::Region as Region>::layers(self.dimensions())
    }

    // Get the whole Texture view (if there is one)
    fn view(&self) -> Option<&wgpu::TextureView> {
        get_specific_view(self, None, None)
    }

    // Get the internally stored views
    // Starts with the base (whole) texture view, then iterates over the mip levels
    fn views(&self) -> Option<&[wgpu::TextureView]>;

    // Get the view of a specific layer of the whole texture (if enabled and if there is one)
    fn layer_view(&self, layer: u32) -> Option<&wgpu::TextureView>
    where
        <Self::Region as Region>::O: LayeredOrigin,
    {
        get_specific_view(self, Some(layer), None)
    }

    // Get the mip levels of the texture immutably
    // Doesn't include the "whole" texture view
    fn mips(&self) -> MipLevelsRef<Self> {
        MipLevelsRef { texture: self }
    }

    // Get the mip levels of the texture mutably
    // Doesn't include the "whole" texture view
    fn mips_mut(&mut self) -> MipLevelsMut<Self> {
        MipLevelsMut {
            texture: self,
            mutated: Cell::new(0),
            borrowed: Cell::new(0),
        }
    }

    // Try to use the whole texture as a renderable target. This will fail if the texture isn't supported as render target
    // or if it's dimensions don't correspond to a 2D image
    fn as_render_target(&mut self) -> Result<RenderTarget<Self::T>, TextureAsTargetError> {
        if !self.usage().contains(TextureUsage::TARGET) {
            return Err(TextureAsTargetError::MissingTargetUsage);
        }

        if self.mips().len() > 1 {
            return Err(TextureAsTargetError::TextureMultipleMips);
        }

        if !self.region().can_render_to_mip() {
            return Err(TextureAsTargetError::RegionIsNot2D);
        }

        Ok(RenderTarget {
            _phantom: PhantomData,
            view: self.view().unwrap(),
        })
    }

    // Uses a specific layer of a texture as a renderable target. This will fail if the texture isn't supported as render target
    // or if it's dimensions don't correspond to a 2D image
    fn layer_as_render_target(
        &mut self,
        layer: u32,
    ) -> Result<RenderTarget<Self::T>, TextureAsTargetError>
    where
        <Self::Region as Region>::O: LayeredOrigin,
    {
        if !self.usage().contains(TextureUsage::TARGET) {
            return Err(TextureAsTargetError::MissingTargetUsage);
        }

        if self.mips().len() > 1 {
            return Err(TextureAsTargetError::TextureMultipleMips);
        }

        if self.dimensions().depth_or_layers() > 1 {
            return Err(TextureAsTargetError::RegionIsNot2D);
        }

        Ok(RenderTarget {
            _phantom: PhantomData,
            view: self.layer_view(layer).unwrap(),
        })
    }

    // Tries to resize the texture to a new size, whilst clearing the contents
    // Mipmapping currently not supported and multi-layer texture currently not supported
    fn resize(&mut self, extent: <Self::Region as Region>::E) -> Result<(), TextureResizeError> {
        let graphics = self.graphics();

        if self.levels() > 1 {
            return Err(TextureResizeError::MipMappingUnsupported);
        }

        if <Self::Region as Region>::is_multi_layered() {
            return Err(TextureResizeError::LayeredUnsupported);
        }

        if !extent.is_valid() {
            return Err(TextureResizeError::InvalidExtent);
        }

        if !size_within_limits::<Self::Region>(&graphics, extent) {
            return Err(TextureResizeError::ExtentLimit);
        }

        if self.mode() != TextureMode::Resizable {
            return Err(TextureResizeError::NotResizable);
        }

        // Same size; not going to resize
        if extent == self.dimensions() {
            return Ok(());
        }

        // Fetch dimensions, name, and texel format
        let view_dimensions = <Self::Region as Region>::view_dimension();
        let dimension = <Self::Region as Region>::dimension();
        let name = utils::pretty_type_name::<Self::T>();
        let format = <Self::T>::format();
        log::debug!(
            "Resizing texture, {dimension:?}, <{name}>, {}x{}x{}",
            extent.width(),
            extent.height(),
            extent.depth_or_layers()
        );

        // Config for the Wgpu texture
        let descriptor = TextureDescriptor {
            size: extent_to_extent3d::<Self::Region>(extent),
            mip_level_count: 1,
            sample_count: 1,
            dimension,
            format,
            usage: texture_usages(self.usage()),
            label: None,
            view_formats: &[],
        };

        // Create a new texture
        let texture = graphics.device().create_texture(&descriptor);

        // Get color texture aspect for the texture view and ImageCopyTexture
        let aspect = texture_aspect::<Self::T>();

        // Create the texture's texture view descriptor
        let view_descriptor = TextureViewDescriptor {
            format: Some(format),
            dimension: Some(view_dimensions),
            aspect,
            ..Default::default()
        };

        self.uncache();

        // Create an texture view of the whole texture
        let view = texture.create_view(&view_descriptor);
        assert_eq!(self.views().map(|x| x.len()).unwrap_or(1), 1);
        unsafe {
            self.replace_raw_parts(texture, Some(vec![view]), extent);
        }

        Ok(())
    }

    // Get the stored graphics context
    fn graphics(&self) -> Graphics;

    // Create a texture struct from it's raw components
    // This will simply create the texture struct, and it assumes
    // that the texture was already created externally with the right parameters
    unsafe fn from_raw_parts(
        graphics: &Graphics,
        texture: wgpu::Texture,
        views: Option<Vec<wgpu::TextureView>>,
        sampler: Option<Arc<wgpu::Sampler>>,
        sampling: Option<SamplerSettings>,
        dimensions: <Self::Region as Region>::E,
        usage: TextureUsage,
        mode: TextureMode,
    ) -> Self;

    // Frees the texture from the cached shader data (called when dropped)
    // If the texture is still used this will practically do nothing (it will remove bind group and regenerate it)
    fn uncache(&self) {
        if let Some(views) = self.views() {
            for view in views {
                let id = crate::Id::new(view.global_id(), crate::IdVariant::TextureView);
                self.graphics().drop_cached_bind_group_resource(id);
            }
        }
    }

    // Called when the user resizes the texture
    // Should not be called manually.
    unsafe fn replace_raw_parts(
        &mut self,
        texture: wgpu::Texture,
        views: Option<Vec<wgpu::TextureView>>,
        dimensions: <Self::Region as Region>::E,
    );
}

// Get the view of a specific mip level and specific layer
// If either params are None, means that we must get the view that corrresponds to "all" layers/mips
pub(crate) fn get_specific_view<T: Texture>(
    texture: &T,
    layer: Option<u32>,
    level: Option<u32>,
) -> Option<&wgpu::TextureView> {
    let max_layers = texture.layers();
    let max_levels = texture.levels();

    // Return if the layer index is invalid
    if let Some(layer) = layer {
        if layer >= max_layers {
            return None;
        }
    }

    // Return if the level index is invalid
    if let Some(level) = level {
        if level >= max_levels {
            return None;
        }
    }

    // all mips, all layers
    // all mips, first layer
    // all mips, second layer
    // first mip, all layers
    // first mip, first layer
    // first mip, second layer
    // second mip, all layers
    // second mip, first layer
    // second mip, second  layer
    let views = texture.views()?;
    let offset = layer
        .map(|x| x + (max_layers - 1).min(1))
        .unwrap_or_default();
    let base = level
        .map(|x| x + (max_levels - 1).min(1))
        .unwrap_or_default();

    let index = base * max_layers + offset;
    let index = index as usize;
    views.get(index)
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
        extent.depth_or_layers(),
    );

    Ok(levels)
}

// Creates the textures views for the base texture and it's corresponding mips
// This will also handles layered textures and layered mip mapped textures
// In the case of layered texture, this will have the following format
// TODO: Implement some sort of setting to know what views we actually need
// all mips, all layers
// all mips, first layer
// all mips, second layer
// first mip, all layers
// first mip, first layer
// first mip, second layer
// second mip, all layers
// second mip, first layer
// second mip, second  layer
pub(crate) fn create_texture_views<T: Texel, R: Region>(
    texture: &wgpu::Texture,
    format: wgpu::TextureFormat,
    extent: R::E,
    levels: u32,
    layers: u32,
) -> Vec<wgpu::TextureView> {
    let aspect = texture_aspect::<T>();
    let mut views = Vec::new();
    let levels = (levels > 1).then_some(levels + 1).unwrap_or(levels);
    let layers = (layers > 1).then_some(layers + 1).unwrap_or(layers);

    log::debug!(
        "Creating level views for texture (max = {levels}) with extent {}x{}x{}",
        extent.width(),
        extent.height(),
        extent.depth_or_layers(),
    );

    for level in 0..levels {
        // Check if we should create a view with all mips
        let level = (level > 0).then(|| level - 1);

        for layer in 0..layers {
            // Check if we should create a view with all layers
            let layer = (layer > 0).then(|| layer - 1);

            // Modify the dimension view based on the current layer
            let dimension = match (layer, R::view_dimension()) {
                (Some(_), wgpu::TextureViewDimension::D2Array) => wgpu::TextureViewDimension::D2,
                (Some(_), wgpu::TextureViewDimension::CubeArray) => {
                    wgpu::TextureViewDimension::Cube
                }
                (Some(_), wgpu::TextureViewDimension::Cube) => wgpu::TextureViewDimension::D2,
                (None, x) => x,
                _ => panic!("Not supported"),
            };

            // Create the texture's texture view descriptor
            let desc = TextureViewDescriptor {
                format: Some(format),
                dimension: Some(dimension),
                aspect,
                base_mip_level: level.unwrap_or_default(),
                mip_level_count: level.map(|_| 1),
                base_array_layer: layer.unwrap_or_default(),
                array_layer_count: layer.map(|_| 1),
                ..Default::default()
            };

            // Create an texture view of the whole texture
            views.push(texture.create_view(&desc));
        }
    }

    views
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
    graphics.staging_pool().write_texture(
        &graphics,
        image_copy_texture,
        image_data_layout,
        extent_to_extent3d::<R>(extent),
        bytes,
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
    write: impl AsMut<MipLevelMut<'a, T>>,
    read: impl AsRef<MipLevelRef<'a, O>>,
    src_subregion: Option<O::Region>,
    dst_subregion: Option<T::Region>,
) {
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
        depth_or_array_layers: R::depth_or_layers(dimensions),
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
        crate::TexelChannels::Depth => wgpu::TextureAspect::DepthOnly,
        crate::TexelChannels::Stencil => wgpu::TextureAspect::StencilOnly,
        _ => wgpu::TextureAspect::All,
    }
}
