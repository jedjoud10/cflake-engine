use std::{mem::transmute, num::NonZeroU32, sync::Arc};

use smallvec::SmallVec;
use wgpu::{TextureDescriptor, TextureViewDescriptor, SamplerDescriptor};

use crate::{
    Extent, Graphics, MipLevelMut, MipLevelRef, Region,
    Texel, TextureInitializationError, TextureMipLayerError,
    TextureMode, TextureSamplerError, TextureUsage, SamplerSettings, SamplerWrap, TextureMipMaps, Sampler, RenderTarget, TextureAsTargetError,
};

// Possibly predefined texel data
type Texels<'a, T> = Option<&'a [<T as Texel>::Storage]>;

// A texture is an abstraction over Vulkan images to allow us to access/modify them with ease
// A texture is a container of multiple texels (like pixels, but for textures) that are stored on the GPU
// This trait is implemented for all variants of textures (1D, 2D, 3D, Layered)
pub trait Texture: Sized {
    // Texel region (position + extent)
    type Region: Region;

    // Texel layout that we will use internally
    type T: Texel;

    // Create a new texture with some possibly predefined data
    fn from_texels(
        graphics: &Graphics,
        texels: Texels<Self::T>,
        dimensions: <Self::Region as Region>::E,
        mode: TextureMode,
        usage: TextureUsage,
        sampling: SamplerSettings,
        mipmaps: TextureMipMaps<Self::T>,
    ) -> Result<Self, TextureInitializationError> {
        let format = <Self::T as Texel>::format();
        let channels = <Self::T as Texel>::channels();
        let bytes_per_channel = <Self::T as Texel>::bytes_per_channel();
        let bytes_per_texel = bytes_per_channel as u64 * channels.count() as u64;

        // Make sure the number of texels matches up with the dimensions
        if let Some(texels) = texels {
            if dimensions.area() as usize != texels.len() {
                return Err(
                    TextureInitializationError::TexelDimensionsMismatch(
                        texels.len(),
                        dimensions.width(),
                        dimensions.height(),
                        dimensions.depth(),
                    ),
                );
            }
        }

        // Get the image type using the dimensionality
        let dimension = <<Self::Region as Region>::E as Extent>::dimensionality();
        let extent = wgpu::Extent3d {
            width: dimensions.width(),
            height: dimensions.height(),
            depth_or_array_layers: dimensions.depth(),
        };

        // Get optimal texture usage
        let usages = wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT;

        // Check if the format is valid for the given usage flag
        let texture_format_features = graphics.adapter().get_texture_format_features(format);
        if !texture_format_features.allowed_usages.contains(usages) {
            return Err(TextureInitializationError::FormatNotSupported(format))
        }

        // Don't use mipmapping with NPOT textures
        if let TextureMipMaps::Disabled = mipmaps {} else {
            if !dimensions.is_power_of_two() {
                return Err(TextureInitializationError::MipMapGenerationNPOT);
            }

            panic!();
        }

        // Config for the Wgpu texture
        let descriptor = TextureDescriptor {
            size: extent,
            mip_level_count: 1,
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
        log::debug!("Creating texture, {dimension:?}, <{name}>, {}x{}x{}", dimensions.width(), dimensions.height(), dimensions.depth());

        // Fetch a new sampler for the given sampling settings
        let sampler = crate::get_or_insert_sampler(graphics, sampling); 

        // Convert the texels to bytes
        let bytes = texels.map(|texels| {
            bytemuck::cast_slice::<<Self::T as Texel>::Storage, u8>(
                texels,
            )
        });

        // Get color texture aspect for the texture view and ImageCopyTexture
        let aspect = match <Self::T as Texel>::channels() {
            crate::ChannelsType::Vector(_) => wgpu::TextureAspect::All,
            crate::ChannelsType::Depth => wgpu::TextureAspect::DepthOnly,
            crate::ChannelsType::Stencil => wgpu::TextureAspect::StencilOnly,
        };

        // Fill the texture with the appropriate data
        if let Some(bytes) = bytes {
            // Bytes per row of texel data
            let bytes_per_row =
                NonZeroU32::new(bytes_per_texel as u32 * dimensions.width());

            // FIXME: Does this work with 3D textures?
            let image_data_layout = wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row,
                rows_per_image: NonZeroU32::new(dimensions.height()),
            };

            let image_copy_texture = wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            };

            // TODO: DO this shit but with mip mapped textures too
            graphics.queue().write_texture(
                image_copy_texture,
                bytes,            
                image_data_layout,
                extent,
            );
        }

        // Create the texture's texture view descriptor
        let view_descriptor = TextureViewDescriptor {
            format: Some(format),
            dimension: Some(match dimension {
                wgpu::TextureDimension::D1 => wgpu::TextureViewDimension::D1,
                wgpu::TextureDimension::D2 => wgpu::TextureViewDimension::D2,
                wgpu::TextureDimension::D3 => wgpu::TextureViewDimension::D3,
            }),
            aspect,
            ..Default::default()
        };

        // Create an texture view of the whole texture
        // TODO: Create MULTIPLE views for the texture
        let view = texture.create_view(&view_descriptor);
        let views =
            SmallVec::from_buf([view]);

        Ok(unsafe {
            Self::from_raw_parts(
                graphics, texture, views, sampler, sampling, dimensions, usage, mode,
            )
        })
    }

    // Get the texture's region (origin state is default)
    fn region(&self) -> Self::Region {
        Self::Region::with_extent(self.dimensions())
    }

    // Checks if we can access a region of the texture
    fn is_region_valid(&self, region: Self::Region) -> bool {
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

    // Get the underlying WGPU Texture
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
    ) -> Result<MipLevelRef<Self>, TextureMipLayerError> {
        todo!()
    }

    // Get a single mip level from the texture, mutably (uses internal mutability pattern)
    fn mip_mut(
        &mut self,
        _level: u8,
    ) -> Result<MipLevelMut<Self>, TextureMipLayerError> {
        todo!()
    }

    // Use the whole texture as a render target
    fn as_render_target(&mut self) -> Result<RenderTarget<Self::T>, TextureAsTargetError> {
        todo!()
    }

    // Try to use the texture as a renderable target
    // This will fail if the texture isn't supported

    // Create a texture struct from it's raw components
    // This will simply create the texture struct, and it assumes
    // that the texture was already created externally
    unsafe fn from_raw_parts(
        graphics: &Graphics,
        texture: wgpu::Texture,
        views: SmallVec<[wgpu::TextureView; 1]>,
        sampler: Arc<wgpu::Sampler>,
        sampling: SamplerSettings,
        dimensions: <Self::Region as crate::Region>::E,
        usage: TextureUsage,
        mode: TextureMode,
    ) -> Self;
}