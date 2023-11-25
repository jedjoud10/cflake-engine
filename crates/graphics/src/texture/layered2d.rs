use std::{marker::PhantomData, mem::ManuallyDrop, sync::Arc, time::Instant};

use assets::Asset;
use smallvec::SmallVec;

use crate::{format::{Texel, ImageTexel}, context::Graphics};
use super::{
    Extent, Sampler, SamplerSettings, Texture, TextureAssetLoadError,
    TextureInitializationError, TextureMipMaps, TextureUsage, TextureViewSettings, RawTexels,
};

// A layered 2D texture that contains multiple texels that are stored in multiple layers
// Each texel can be either a single value, RG, RGB, or even RGBA
pub struct LayeredTexture2D<T: Texel> {
    // Raw WGPU
    texture: wgpu::Texture,
    views: Vec<(wgpu::TextureView, TextureViewSettings)>,

    // Main texture settings
    dimensions: (vek::Extent2<u32>, u32),

    // Permissions
    usage: TextureUsage,
    _phantom: PhantomData<T>,

    // Keep the graphics API alive
    graphics: Graphics,
}

impl<T: Texel> Texture for LayeredTexture2D<T> {
    type Region = ((vek::Vec2<u32>, u32), (vek::Extent2<u32>, u32));
    type T = T;

    fn dimensions(&self) -> <Self::Region as super::Region>::E {
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

    fn graphics(&self) -> Graphics {
        self.graphics.clone()
    }

    unsafe fn from_raw_parts(
        graphics: &Graphics,
        texture: wgpu::Texture,
        views: Vec<(wgpu::TextureView, TextureViewSettings)>,
        dimensions: (vek::Extent2<u32>, u32),
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
}

// Combine multiple raw texels into a single LayeredTexture2D texture
// Returns None if the textures don't have the same size or if we can't read them
pub fn combine_into_layered<T: Texel + ImageTexel>(
    graphics: &Graphics,
    raw: Vec<RawTexels<T>>,
    sampling: Option<SamplerSettings>,
    mipmaps: TextureMipMaps<T>,
    views: &[TextureViewSettings],
    usage: TextureUsage,
) -> Option<LayeredTexture2D<T>> {
    // Can't have shit in ohio
    if raw.is_empty() {
        return None;
    }

    // Make sure the textures are the same size
    let dimensions = raw[0].dimensions();
    if raw.iter().any(|tex| tex.dimensions() != dimensions) {
        return None;
    }

    // Get the (packed) texels from the textures
    let texels = raw
        .iter()
        .flat_map(|raw| raw.texels().iter().cloned())
        .collect::<Vec<_>>();

    // Check if we must generate mip maps
    let generate_mip_maps = if let TextureMipMaps::Manual { mips: &[] } = mipmaps {
        true
    } else {
        false
    };

    let extent = (dimensions, raw.len() as u32);

    // Generate each mip's texel data
    let mips = if generate_mip_maps {
        Some(
            super::generate_mip_map::<T, ((vek::Vec2<u32>, u32), (vek::Extent2<u32>, u32))>(
                &texels, extent,
            )
            .ok_or(TextureInitializationError::MipMapGenerationNPOT)
            .unwrap(),
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
        mipmaps
    };

    Some(
        LayeredTexture2D::from_texels(
            graphics,
            Some(&texels),
            extent,
            usage,
            views,
            mipmaps,
        )
        .unwrap(),
    )
}
