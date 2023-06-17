use std::{marker::PhantomData, mem::ManuallyDrop, sync::Arc, time::Instant};

use assets::Asset;
use smallvec::SmallVec;

use crate::{
    Extent, Graphics, ImageTexel, Sampler, SamplerSettings, Texel, Texture, TextureAssetLoadError,
    TextureInitializationError, TextureMipMaps, TextureUsage, TextureViewSettings,
};

// A #D texture that contains multiple texels that have their own channels
// Each texel can be either a single value, RG, RGB, or even RGBA
pub struct Texture3D<T: Texel> {
    // Raw WGPU
    texture: wgpu::Texture,
    views: Vec<wgpu::TextureView>,

    // Main texture settings
    dimensions: vek::Extent3<u32>,

    // Permissions
    usage: TextureUsage,
    _phantom: PhantomData<T>,

    // Shader Sampler
    sampler: Option<Arc<wgpu::Sampler>>,
    sampling: Option<SamplerSettings>,

    // Keep the graphics API alive
    graphics: Graphics,
}

impl<T: Texel> Texture for Texture3D<T> {
    type Region = (vek::Vec3<u32>, vek::Extent3<u32>);
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

    fn raw_views(&self) -> &[wgpu::TextureView] {
        &self.views
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
        views: Vec<wgpu::TextureView>,
        sampler: Option<Arc<wgpu::Sampler>>,
        sampling: Option<SamplerSettings>,
        dimensions: vek::Extent3<u32>,
        usage: TextureUsage,
    ) -> Self {
        Self {
            texture,
            views,
            dimensions,
            usage,
            _phantom: PhantomData,
            graphics: graphics.clone(),
            sampler,
            sampling,
        }
    }
}

impl<T: Texel> Drop for Texture3D<T> {
    fn drop(&mut self) {
        self.uncache();
    }
}
