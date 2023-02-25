use std::{
    marker::PhantomData, mem::ManuallyDrop, sync::Arc, time::Instant,
};

use assets::Asset;
use naga::Sampling;
use smallvec::SmallVec;

use crate::{
    Graphics, ImageTexel, Sampler, SamplerSettings, Texel, Texture,
    TextureAssetLoadError, TextureMipMaps, TextureMode, TextureUsage, Extent, TextureInitializationError,
};

// A 2D texture that contains multiple texels that have their own channels
// Each texel can be either a single value, RG, RGB, or even RGBA
pub struct Texture2D<T: Texel> {
    // Raw WGPU
    texture: wgpu::Texture,
    views: SmallVec<[wgpu::TextureView; 1]>,

    // Main texture settings
    dimensions: vek::Extent2<u32>,

    // Permissions
    usage: TextureUsage,
    mode: TextureMode,
    _phantom: PhantomData<T>,

    // Shader Sampler
    sampler: Arc<wgpu::Sampler>,
    sampling: SamplerSettings,

    // Keep the graphics API alive
    graphics: Graphics,
}

impl<T: Texel> super::raw::RawTexture<(vek::Vec2<u32>, vek::Extent2<u32>)> for Texture2D<T> {
    fn graphics(&self) -> Graphics {
        self.graphics.clone()
    }

    unsafe fn from_raw_parts(
        graphics: &Graphics,
        texture: wgpu::Texture,
        views: SmallVec<[wgpu::TextureView; 1]>,
        sampler: Arc<wgpu::Sampler>,
        sampling: SamplerSettings,
        dimensions: vek::Extent2<u32>,
        usage: TextureUsage,
        mode: TextureMode,
    ) -> Self {
        Self {
            texture,
            views,
            dimensions,
            usage,
            mode,
            _phantom: PhantomData,
            graphics: graphics.clone(),
            sampler,
            sampling,
        }
    }

    unsafe fn replace_raw_parts(
        &mut self,
        texture: wgpu::Texture,
        views: SmallVec<[wgpu::TextureView; 1]>,
        dimensions: vek::Extent2<u32>,
    ) {
        self.texture = texture;
        self.views = views;
        self.dimensions = dimensions;
    }
}

impl<T: Texel> Texture for Texture2D<T> {
    type Region = (vek::Vec2<u32>, vek::Extent2<u32>);
    type T = T;

    fn dimensions(&self) -> <Self::Region as crate::Region>::E {
        self.dimensions
    }

    fn mode(&self) -> TextureMode {
        self.mode
    }

    fn usage(&self) -> TextureUsage {
        self.usage
    }

    fn raw(&self) -> &wgpu::Texture {
        &self.texture
    }

    fn views(&self) -> &[wgpu::TextureView] {
        &self.views
    }

    fn sampler(&self) -> Sampler<Self::T> {
        Sampler {
            sampler: &self.sampler,
            _phantom: PhantomData,
            settings: &self.sampling,
        }
    }
}

// Texture settings that we shall use when loading in a new texture
#[derive(Clone, Copy)]
pub struct TextureImportSettings<'m, T: Texel> {
    pub sampling: SamplerSettings,
    pub mode: TextureMode,
    pub usage: TextureUsage,
    pub mipmaps: TextureMipMaps<'m, 'm, T>,
}

impl<T: Texel> Default for TextureImportSettings<'_, T> {
    fn default() -> Self {
        Self {
            sampling: SamplerSettings::default(),
            mode: TextureMode::default(),
            usage: TextureUsage::default(),
            mipmaps: TextureMipMaps::Manual { mips: &[] },
        }
    }
}

impl<T: ImageTexel> Asset for Texture2D<T> {
    type Context<'ctx> = Graphics;
    type Settings<'stg> = TextureImportSettings<'stg, T>;
    type Err = TextureAssetLoadError;

    fn extensions() -> &'static [&'static str] {
        &["png", "jpg", "jpeg"]
    }

    fn deserialize<'c, 's>(
        data: assets::Data,
        graphics: Self::Context<'c>,
        settings: Self::Settings<'s>,
    ) -> Result<Self, Self::Err> {
        let i = Instant::now();

        // Load the texture using the Image crate
        let image = image::load_from_memory(data.bytes())
            .map_err(TextureAssetLoadError::ImageError)?;
        log::debug!(
            "Took {:?} to deserialize texture {:?}",
            i.elapsed(),
            data.path()
        );

        // Get 2D dimensions and texel data 
        let dimensions =
            vek::Extent2::new(image.width(), image.height());
        let texels = T::to_image_texels(image);

        // Check if we must generate mip maps
        if let TextureMipMaps::Manual { mips: &[] } = settings.mipmaps {
            // Generate each mip's texel data
            let mips = super::generate_mip_map::<T, vek::Extent2<u32>>(
                &texels,
                dimensions
            ).ok_or(TextureInitializationError::MipMapGenerationNPOT)?;
        
            
        }

        // Create the texture
        Self::from_texels(
            &graphics,
            Some(&texels),
            dimensions,
            settings.mode,
            settings.usage,
            settings.sampling,
            settings.mipmaps,
        ).map_err(TextureAssetLoadError::Initialization)
    }
}
