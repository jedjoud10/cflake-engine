
use std::{mem::transmute, num::{NonZeroU32, NonZeroU8}, sync::Arc, marker::PhantomData};

use smallvec::SmallVec;
use wgpu::{
    SamplerDescriptor, TextureDescriptor, TextureViewDescriptor,
};

use crate::{
    Extent, Graphics, MipLevelMut, MipLevelRef, Region, RenderTarget,
    Sampler, SamplerSettings, SamplerWrap, Texel,
    TextureAsTargetError, TextureInitializationError,
    TextureMipLevelError, TextureMipMaps, TextureMode,
    TextureSamplerError, TextureUsage, TextureResizeError, GpuPodRelaxed, Origin,
};



// Possibly predefined texel data
type Texels<'a, T> = Option<&'a [<T as Texel>::Storage]>;

// A texture is an abstraction over Vulkan images to allow us to access/modify them with ease
// A texture is a container of multiple texels (like pixels, but for textures) that are stored on the GPU
// This trait is implemented for all variants of textures (1D, 2D, 3D, Layered)
pub trait Texture: Sized + raw::RawTexture<Self::Region> {
    // Texel region (position + extent)
    type Region: Region;

    // Texel layout that we will use internally
    type T: Texel;

    // Create a new texture with some possibly predefined data
    fn from_texels(
        graphics: &Graphics,
        texels: Texels<Self::T>,
        extent: <Self::Region as Region>::E,
        mode: TextureMode,
        usage: TextureUsage,
        sampling: SamplerSettings,
        mipmaps: TextureMipMaps<Self::T>,
    ) -> Result<Self, TextureInitializationError> {
        let format = <Self::T as Texel>::format();

        // Make sure the number of texels matches up with the dimensions
        if let Some(texels) = texels {
            if extent.area() as usize != texels.len() {
                return Err(
                    TextureInitializationError::TexelDimensionsMismatch {
                        count: texels.len(),
                        w: extent.width(),
                        h: extent.height(),
                        d: extent.depth(),
                    },
                );
            }
        }

        // Get the image type using the dimensionality
        let dimension =
            <<Self::Region as Region>::E as Extent>::dimensionality();
        let extent_3d = extent_to_extent3d(extent);

        // If the extent contains a 0 in any axii, it's invalid
        if !extent.is_valid() {
            return Err(TextureInitializationError::InvalidExtent);
        }

        // If the extent is greater than the physical limits, it's invalid
        if !size_within_limits(&graphics, extent) {
            return Err(TextureInitializationError::ExtentLimit);
        }

        // Return an error if the texture usage flags are invalid
        if usage.contains(TextureUsage::READ) && !usage.contains(TextureUsage::COPY_SRC) {
            return Err(TextureInitializationError::ReadableWithoutCopySrc);
        } else if usage.contains(TextureUsage::WRITE) && !usage.contains(TextureUsage::COPY_DST) {
            return Err(TextureInitializationError::WritableWithoutCopyDst);
        } else if !usage.contains(TextureUsage::COPY_DST) && (texels.is_some() || match mipmaps {
            TextureMipMaps::Manual { mips } => mips.len() > 0,
            _ => false
        }) {
            return Err(TextureInitializationError::PreinitializedWithoutCopyDst);
        }

        // Get optimal texture usage
        let usages = texture_usages(usage);

        // Check if the format is valid for the given usage flag
        let texture_format_features =
            graphics.adapter().get_texture_format_features(format);
        if !texture_format_features.allowed_usages.contains(usages) {
            return Err(
                TextureInitializationError::FormatNotSupported(
                    format,
                ),
            );
        }

        // Get the number of mip levels for this texture
        let levels = mip_levels(&mipmaps, extent)?;

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
            extent.depth()
        );

        // Fetch a new sampler for the given sampling settings
        let sampler =
            crate::get_or_insert_sampler(graphics, sampling);

        // Get color texture aspect for the texture view and ImageCopyTexture
        let aspect = texture_aspect::<Self::T>();

        // Create a "zeroed" origin
        let origin = <<Self::Region as Region>::O as Default>::default();

        // Always write to the first mip level
        if let Some(texels) = texels {
            write_to_level::<Self::T, Self::Region>(
                origin,
                extent,
                texels,
                &texture,
                aspect,
                0,
                graphics
            );
        }

        // Fill the texture mips with the appropriate data
        match mipmaps {
            TextureMipMaps::Manual { mips } => {
                // Manual mip map generation
                let iter = mips.iter().enumerate().map(|(x, y)| (x + 1, y));
                for (i, texels) in iter {
                    // Downscale the texture extent by two
                    let downscaled_extent = extent.mip_level_dimensions(i as u8); 

                    log::debug!(
                        "Creating manual mip-map layer <{i}> for texture, {dimension:?}, <{name}>, {}x{}x{}",
                        downscaled_extent.width(),
                        downscaled_extent.height(),
                        downscaled_extent.depth()
                    );

                    // Write bytes to the level
                    write_to_level::<Self::T, Self::Region>(
                        origin,
                        downscaled_extent,
                        texels,
                        &texture,
                        aspect,
                        i as u32,
                        graphics
                    );
                }
            },
            _ => {}
        }
        

        // Create the texture's texture view descriptor
        let view_descriptor = TextureViewDescriptor {
            format: Some(format),
            dimension: Some(dims_to_view_dims(dimension)),
            aspect,
            ..Default::default()
        };

        // Create an texture view of the whole texture
        // TODO: Create MULTIPLE views for the texture
        let view = texture.create_view(&view_descriptor);
        let views = SmallVec::from_buf([view]);

        Ok(unsafe {
            Self::from_raw_parts(
                graphics, texture, views, sampler, sampling,
                extent, usage, mode,
            )
        })
    }

    // Get the texture's region (origin state is default)
    fn region(&self) -> Self::Region {
        Self::Region::with_extent(self.dimensions())
    }

    // Checks if we can access a region of the texture
    fn is_region_accessible(&self, region: Self::Region) -> bool {
        let extent = <Self::Region as Region>::extent_from_origin(
            region.origin(),
        ) + region.extent();
        self.dimensions().is_larger_than(extent)
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
    fn sampler(&self) -> Sampler<Self::T>;

    // Get the underlying Texture view
    fn view(&self) -> &wgpu::TextureView {
        &self.views()[0]
    }

    // Get all the allocated texture views
    fn views(&self) -> &[wgpu::TextureView];

    // Get a single mip level from the texture, immutably
    fn mip(
        &self,
        _level: u8,
    ) -> Result<MipLevelRef<Self>, TextureMipLevelError> {
        todo!()
    }

    // Get a single mip level from the texture, mutably (uses internal mutability pattern)
    fn mip_mut(
        &mut self,
        _level: u8,
    ) -> Result<MipLevelMut<Self>, TextureMipLevelError> {
        todo!()
    }

    // Try to use the texture as a renderable target. This will fail if the texture isn't supported as render target 
    // or if it has mipmapping on (the user can still use each mip layer as individual render targets though)
    fn as_render_target(
        &mut self,
    ) -> Result<RenderTarget<Self::T>, TextureAsTargetError> {
        Ok(RenderTarget {
            _phantom: PhantomData,
            view: self.view()
        })
    }

    // Tries to resize the texture to a new size, whilst clearing the contents
    // Mipmapping currently no supported
    fn resize(&mut self, extent: <Self::Region as Region>::E) -> Result<(), TextureResizeError> {
        let graphics = self.graphics();
        if self.views().len() > 1 {
            return Err(TextureResizeError::MipMappingUnsupported);
        }

        if !extent.is_valid() {
            return Err(TextureResizeError::InvalidExtent);
        }

        if !size_within_limits(&graphics, extent) {
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
        let dimension = <<Self::Region as Region>::E as Extent>::dimensionality();
        let name = utils::pretty_type_name::<Self::T>();
        let format = <Self::T>::format();
        log::debug!(
            "Resizing texture, {dimension:?}, <{name}>, {}x{}x{}",
            extent.width(),
            extent.height(),
            extent.depth()
        );

        // Config for the Wgpu texture
        let descriptor = TextureDescriptor {
            size: extent_to_extent3d(extent),
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
            dimension: Some(dims_to_view_dims(dimension)),
            aspect,
            ..Default::default()
        };

        // Create an texture view of the whole texture
        let view = texture.create_view(&view_descriptor);
        let views = SmallVec::from_buf([view]);
        unsafe {
            self.replace_raw_parts(texture, views, extent);
        }

        Ok(())
    }
}

// Get the number of mip levels that the texture should use
fn mip_levels<T: Texel, E: Extent>(
    mipmaps: &TextureMipMaps<T>,
    extent: E,
) -> Result<u32, TextureInitializationError> {
    let max_mip_levels = if matches!(mipmaps, TextureMipMaps::Disabled) {
        1u8 as u32
    } else {
        let max = extent.levels().ok_or(TextureInitializationError::MipMapGenerationNPOT)?;
        max.get() as u32
    };

    // Convert Auto to Zeroed (since if this texture was loaded from disk, it would've been Manual instead)

    let levels = match *mipmaps {
        // Automatic mip level generation, but fills mips with zeroes  
        TextureMipMaps::Zeroed { clamp } => {
            let max = clamp.map(|x| x.get()).unwrap_or(u8::MAX);
            (max as u32).min(max_mip_levels)
        },

        // Manual mip level generatation, but fills mips with corresponding data 
        TextureMipMaps::Manual { mips } => {
            let levels = mips.len() as u32 + 1;
            levels.min(max_mip_levels)
        },

        // No mip maps
        TextureMipMaps::Disabled => 1,
    };

    log::debug!("Calculated {levels} mip levels for texture with extent {}x{}x{}",
        extent.width(),
        extent.height(),
        extent.depth()
    );

    Ok(levels)
}

// Write texels to a single level of a texture
// Assumes that the origin and extent are in mip-space
// TODO: Test to check if this works with 3D
pub(crate) fn write_to_level<T: Texel, R: Region>(
    origin: R::O,
    extent: R::E,
    texels: &[T::Storage],
    texture: &wgpu::Texture,
    aspect: wgpu::TextureAspect,
    level: u32,
    graphics: &Graphics,
) {
    let bytes_per_channel = T::bytes_per_channel();
    let bytes_per_texel = bytes_per_channel as u64 * T::channels().count() as u64;
    let extent_3d = extent_to_extent3d(extent);

    // Bytes per row of texel data
    let bytes_per_row = NonZeroU32::new(
        bytes_per_texel as u32 * extent.width(),
    );

    // Convert the texels to bytes
    let bytes = bytemuck::cast_slice::<T::Storage, u8>(
        texels,
    );

    // FIXME: Does this work with 3D textures?
    let image_data_layout = wgpu::ImageDataLayout {
        offset: 0,
        bytes_per_row,
        rows_per_image: None,
    };

    // Create the image copy texture descriptor
    let image_copy_texture = wgpu::ImageCopyTexture {
        texture: texture,
        mip_level: level,
        origin: wgpu::Origin3d::ZERO,
        aspect,
    };

    // Write to the base layer of the texture 
    graphics.queue().write_texture(
        image_copy_texture,
        bytes,
        image_data_layout,
        extent_3d,
    );
}

// Read texels from a single level of a texture
// Assumes that the origin and extent are in mip-space
pub(crate) fn read_from_level<T: Texel, E: Extent, O: Origin>(
    origin: O,
    extent: E,
    texels: &mut [T::Storage],
    texture: &wgpu::Texture,
    aspect: wgpu::TextureAspect,
    level: u32,
    graphics: &Graphics,
) {
}

// Separated this into it's own trait since I didn't want there to be unsafe init/internal functions publicly
pub(crate) mod raw {
    use std::sync::Arc;
    use smallvec::SmallVec;
    use crate::{Region, Graphics, SamplerSettings, TextureUsage, TextureMode};

    pub trait RawTexture<R: Region> {
        // Get the stored graphics context 
        fn graphics(&self) -> Graphics;

        // Create a texture struct from it's raw components
        // This will simply create the texture struct, and it assumes
        // that the texture was already created externally with the right parameters
        unsafe fn from_raw_parts(
            graphics: &Graphics,
            texture: wgpu::Texture,
            views: SmallVec<[wgpu::TextureView; 1]>,
            sampler: Arc<wgpu::Sampler>,
            sampling: SamplerSettings,
            dimensions: <R as Region>::E,
            usage: TextureUsage,
            mode: TextureMode,
        ) -> Self;

        // Replace the underlying raw data with the given new data
        unsafe fn replace_raw_parts(
            &mut self,
            texture: wgpu::Texture,
            views: SmallVec<[wgpu::TextureView; 1]>,
            dimensions: <R as Region>::E,
        );
    }
}

// Check if the given extent is valid within device limits
fn size_within_limits<E: Extent>(graphics: &Graphics, extent: E) -> bool {
    // Create the max possible texture size from device limits
    let limits = graphics.device().limits();
    let max = match E::dimensionality() {
        wgpu::TextureDimension::D1 => limits.max_texture_dimension_1d,
        wgpu::TextureDimension::D2 => limits.max_texture_dimension_2d,
        wgpu::TextureDimension::D3 => limits.max_texture_dimension_3d,
    };
    let max = E::broadcast(max);
        
    // Check if the new texture size is physically possible
    max.is_larger_than(extent)
}

// Convert TextureDimension to TextureViewDimension
fn dims_to_view_dims(dimension: wgpu::TextureDimension) -> wgpu::TextureViewDimension {
    match dimension {
        wgpu::TextureDimension::D1 => {
            wgpu::TextureViewDimension::D1
        }
        wgpu::TextureDimension::D2 => {
            wgpu::TextureViewDimension::D2
        }
        wgpu::TextureDimension::D3 => {
            wgpu::TextureViewDimension::D3
        }
    }
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
pub(crate) fn extent_to_extent3d<E: Extent>(dimensions: E) -> wgpu::Extent3d {
    wgpu::Extent3d {
        width: dimensions.width(),
        height: dimensions.height(),
        depth_or_array_layers: dimensions.depth(),
    }
}

// Get the texture usages from the texture usage wrapper
// Does not check for validity
pub(crate) fn texture_usages(usage: TextureUsage) -> wgpu::TextureUsages {
    let mut usages = wgpu::TextureUsages::empty();

    if usage.contains(TextureUsage::SAMPLED) {
        usages |= wgpu::TextureUsages::TEXTURE_BINDING;
    }

    if usage.contains(TextureUsage::RENDER_TARGET) {
        usages |= wgpu::TextureUsages::RENDER_ATTACHMENT;
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
        crate::TexelChannels::Vector(_)
        | crate::TexelChannels::Srgba { .. } => {
            wgpu::TextureAspect::All
        }
        crate::TexelChannels::Depth => {
            wgpu::TextureAspect::DepthOnly
        }
        crate::TexelChannels::Stencil => {
            wgpu::TextureAspect::StencilOnly
        }
    }
}
