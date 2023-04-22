use std::{marker::PhantomData, mem::ManuallyDrop, sync::Arc, time::Instant};

use assets::Asset;
use smallvec::SmallVec;

use crate::{
    Extent, Graphics, ImageTexel, RawTexels, Sampler, SamplerSettings, Texel, Texture, Texture2D,
    TextureAssetLoadError, TextureInitializationError, TextureMipMaps, TextureMode, TextureUsage,
};

// A layered 2D texture that contains multiple texels that are stored in multiple layers
// Each texel can be either a single value, RG, RGB, or even RGBA
pub struct LayeredTexture2D<T: Texel> {
    // Raw WGPU
    texture: wgpu::Texture,
    views: Option<Vec<wgpu::TextureView>>,

    // Main texture settings
    dimensions: (vek::Extent2<u32>, u32),

    // Permissions
    usage: TextureUsage,
    mode: TextureMode,
    _phantom: PhantomData<T>,

    // Shader Sampler
    sampler: Option<Arc<wgpu::Sampler>>,
    sampling: Option<SamplerSettings>,

    // Keep the graphics API alive
    graphics: Graphics,
}

impl<T: Texel> Texture for LayeredTexture2D<T> {
    type Region = ((vek::Vec2<u32>, u32), (vek::Extent2<u32>, u32));
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

    fn views(&self) -> Option<&[wgpu::TextureView]> {
        self.views.as_ref().map(|x| x.as_slice())
    }

    fn sampler(&self) -> Option<Sampler<Self::T>> {
        self.sampler
            .as_ref()
            .zip(self.sampling.as_ref())
            .map(|(sampler, settings)| Sampler {
                sampler,
                _phantom: PhantomData,
                settings,
            })
    }

    fn graphics(&self) -> Graphics {
        self.graphics.clone()
    }

    unsafe fn from_raw_parts(
        graphics: &Graphics,
        texture: wgpu::Texture,
        views: Option<Vec<wgpu::TextureView>>,
        sampler: Option<Arc<wgpu::Sampler>>,
        sampling: Option<SamplerSettings>,
        dimensions: (vek::Extent2<u32>, u32),
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
        views: Option<Vec<wgpu::TextureView>>,
        dimensions: (vek::Extent2<u32>, u32),
    ) {
        self.texture = texture;
        self.views = views;
        self.dimensions = dimensions;
    }
}

// Combine multiple raw texels into a single LayeredTexture2D texture
// Returns None if the textures don't have the same size or if we can't read them
pub fn combine_into_layered<T: Texel + ImageTexel>(
    graphics: &Graphics,
    raw: Vec<RawTexels<T>>,
    sampling: Option<SamplerSettings>,
    mipmaps: TextureMipMaps<T>,
    mode: TextureMode,
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
            super::generate_mip_map::<T, (vek::Extent2<u32>, u32)>(&texels, extent)
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
            mode,
            usage,
            sampling,
            mipmaps,
        )
        .unwrap(),
    )
}
