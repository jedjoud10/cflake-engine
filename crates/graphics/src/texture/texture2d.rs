use std::{marker::PhantomData, mem::ManuallyDrop, sync::Arc, time::Instant};

use assets::Asset;
use smallvec::SmallVec;

use crate::{
    Extent, Graphics, ImageTexel, RawTexels, Sampler, SamplerSettings, Texel, Texture,
    TextureAssetLoadError, TextureInitializationError, TextureMipMaps, TextureScale, TextureUsage,
    TextureViewDimension, TextureViewSettings,
};

// A 2D texture that contains multiple texels that have their own channels
// Each texel can be either a single value, RG, RGB, or even RGBA
pub struct Texture2D<T: Texel> {
    // Raw WGPU
    texture: wgpu::Texture,
    views: Vec<(wgpu::TextureView, TextureViewSettings)>,

    // Main texture settings
    dimensions: vek::Extent2<u32>,

    // Permissions
    usage: TextureUsage,
    _phantom: PhantomData<T>,

    // Keep the graphics API alive
    graphics: Graphics,
}

impl<T: Texel> Texture for Texture2D<T> {
    type Region = (vek::Vec2<u32>, vek::Extent2<u32>);
    type T = T;

    fn dimensions(&self) -> <Self::Region as crate::Region>::E {
        self.dimensions
    }

    fn usage(&self) -> TextureUsage {
        self.usage
    }

    fn raw(&self) -> &wgpu::Texture {
        &self.texture
    }

    fn raw_views(&self) -> &[(wgpu::TextureView, TextureViewSettings)] {
        &self.views
    }

    unsafe fn from_raw_parts(
        graphics: &Graphics,
        texture: wgpu::Texture,
        views: Vec<(wgpu::TextureView, TextureViewSettings)>,
        dimensions: vek::Extent2<u32>,
        usage: TextureUsage,
    ) -> Self {
        Self {
            texture,
            views,
            dimensions,
            usage,
            _phantom: PhantomData,
            graphics: graphics.clone(),
        }
    }

    fn graphics(&self) -> Graphics {
        self.graphics.clone()
    }
}

// Texture settings that we shall use when loading in a new texture
#[derive(Clone)]
pub struct TextureImportSettings<'s, T: ImageTexel> {
    pub sampling: Option<SamplerSettings>,
    pub usage: TextureUsage,
    pub scale: TextureScale,
    pub views: &'s [TextureViewSettings],
    pub mipmaps: TextureMipMaps<'s, 's, T>,
}

impl<T: ImageTexel> Default for TextureImportSettings<'_, T> {
    fn default() -> Self {
        Self {
            sampling: Some(SamplerSettings::default()),
            scale: TextureScale::Default,
            usage: TextureUsage::SAMPLED | TextureUsage::COPY_DST,
            mipmaps: TextureMipMaps::Manual { mips: &[] },
            views: &[TextureViewSettings {
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
                dimension: TextureViewDimension::D2,
            }],
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
        // Load the raw texels from the file
        let raw = RawTexels::<T>::deserialize(data, (), settings.scale)
            .map_err(TextureAssetLoadError::RawTexelsError)?;

        // Convert raw texels to texture
        texture2d_from_raw(graphics, settings, raw).map_err(TextureAssetLoadError::Initialization)
    }
}

// Load in a texture from the raw texels
pub fn texture2d_from_raw<T: ImageTexel>(
    graphics: Graphics,
    settings: TextureImportSettings<T>,
    raw: RawTexels<T>,
) -> Result<Texture2D<T>, TextureInitializationError> {
    let RawTexels(texels, dimensions) = raw;

    // Check if we must generate mip maps
    let generate_mip_maps = if let TextureMipMaps::Manual { mips: &[] } = settings.mipmaps {
        true
    } else {
        false
    };

    // Generate each mip's texel data
    let mips = if generate_mip_maps {
        Some(
            super::generate_mip_map::<T, (vek::Vec2<u32>, vek::Extent2<u32>)>(&texels, dimensions)
                .ok_or(TextureInitializationError::MipMapGenerationNPOT)?,
        )
    } else {
        None
    };

    // Convert the vecs to slices
    let mips = mips
        .as_ref()
        .map(|mips| mips.iter().map(|x| x.as_slice()).collect::<Vec<_>>());

    // Overwrite the Manual mip map layers if they were empty to begin with
    let mipmaps = if generate_mip_maps {
        TextureMipMaps::Manual {
            mips: &mips.as_ref().unwrap(),
        }
    } else {
        settings.mipmaps
    };

    // Create the texture
    Texture2D::<T>::from_texels(
        &graphics,
        Some(&texels),
        dimensions,
        settings.usage,
        settings.views,
        mipmaps,
    )
}
