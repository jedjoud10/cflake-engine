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

// A #D texture that contains multiple texels that have their own channels
// Each texel can be either a single value, RG, RGB, or even RGBA
pub struct Texture3D<T: Texel> {
    // Raw WGPU
    texture: wgpu::Texture,
    views: SmallVec<[wgpu::TextureView; 1]>,

    // Main texture settings
    dimensions: vek::Extent3<u32>,

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

impl<T: Texel>
    super::raw::RawTexture<(vek::Vec3<u32>, vek::Extent3<u32>)>
    for Texture3D<T>
{
    fn graphics(&self) -> Graphics {
        self.graphics.clone()
    }

    unsafe fn from_raw_parts(
        graphics: &Graphics,
        texture: wgpu::Texture,
        views: SmallVec<[wgpu::TextureView; 1]>,
        sampler: Arc<wgpu::Sampler>,
        sampling: SamplerSettings,
        dimensions: vek::Extent3<u32>,
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
        dimensions: vek::Extent3<u32>,
    ) {
        self.texture = texture;
        self.views = views;
        self.dimensions = dimensions;
    }
}

impl<T: Texel> Texture for Texture3D<T> {
    type Region = (vek::Vec3<u32>, vek::Extent3<u32>);
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