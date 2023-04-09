use std::{
    marker::PhantomData, mem::ManuallyDrop, sync::Arc, time::Instant,
};

use assets::Asset;
use smallvec::SmallVec;

use crate::{
    Extent, Graphics, ImageTexel, Sampler, SamplerSettings, Texel,
    Texture, TextureAssetLoadError, TextureInitializationError,
    TextureMipMaps, TextureMode, TextureUsage,
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
        self.sampler.as_ref().zip(self.sampling.as_ref()).map(|(sampler, settings)| Sampler {
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