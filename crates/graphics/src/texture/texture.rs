use std::{mem::transmute, num::NonZeroU32};

use wgpu::{TextureDescriptor, TextureViewDescriptor};

use crate::{
    Extent, Graphics, MipLevelMut, MipLevelRef, Region, Sampler,
    Texel, TextureInitializationError, TextureMipLayerError,
    TextureMode, TextureSamplerError, TextureUsage,
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
        let dimension = match <<Self::Region as Region>::E as Extent>::dimensionality() {
            1 => wgpu::TextureDimension::D1,
            2 => wgpu::TextureDimension::D2,
            3 => wgpu::TextureDimension::D3,
            _ => panic!("1D, 2D, or 3D textures are the only supported types of textures")
        };
        let extent = wgpu::Extent3d {
            width: dimensions.width(),
            height: dimensions.height(),
            depth_or_array_layers: dimensions.depth(),
        };

        // Get optimal texture usage
        let usages = wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_DST;

        // TODO: Check if the format is valid for the given usage flag

        // TODO: Don't use mipmapping with NPOT textures

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

        // Convert the texels to bytes
        let bytes = texels.map(|texels| {
            bytemuck::cast_slice::<<Self::T as Texel>::Storage, u8>(
                texels,
            )
        });

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

            graphics.queue().write_texture(
                image_copy_texture,
                bytes,
                image_data_layout,
                extent,
            );
        }

        // Create an texture view of the whole texture
        let view =
            texture.create_view(&TextureViewDescriptor::default());

        Ok(unsafe {
            Self::from_raw_parts(
                graphics, texture, view, dimensions, usage, mode,
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
    fn texture(&self) -> &wgpu::Texture;

    // Get the underlying Texture view
    fn view(&self) -> &wgpu::TextureView;

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

    // Try to get a sampler for this whole texture so we can read from it within shaders
    fn as_sampler(
        &self,
    ) -> Result<Sampler<Self>, TextureSamplerError> {
        todo!()
    }

    // Create a texture struct from it's raw components
    // This will simply create the texture struct, and it assumes
    // that the texture was already created externally
    unsafe fn from_raw_parts(
        graphics: &Graphics,
        texture: wgpu::Texture,
        view: wgpu::TextureView,
        dimensions: <Self::Region as Region>::E,
        usage: TextureUsage,
        mode: TextureMode,
    ) -> Self;
}
