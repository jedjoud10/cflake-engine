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

// A cubemap texture that contains multiple texels that have their own channels
// A cubemap contains 6 base layer, that represent each side of the cube
// TODO: TEST
pub struct CubeMap<T: Texel> {
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

impl<T: Texel>
    super::raw::RawTexture<((vek::Vec2<u32>, u32), vek::Extent2<u32>)>
    for CubeMap<T>
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

impl<T: Texel> Texture for CubeMap<T> {
    type Region = ((vek::Vec2<u32>, u32), vek::Extent2<u32>);
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

// Convolution / unwrapping mode that we can use to load in cubemaps from equirectangular textures
#[derive(Default, Clone, Copy, PartialEq)]
pub enum CubeMapUnwrapMode {
    // Simply load the cubemap as an equirectangular texture, no convolution applied
    #[default]
    Equirectangular,
    
    // Convolute the cubemap into an environment map that we can use for diffuse lighting
    DiffuseIrradiance,

    // Convolute the cubemap for usage within a specular IBL
    // This requires the cubemap settings to have mipmap enabled
    SpecularIBL,
}

// Cubemap settings that we shall use when loading in a new cubemap
#[derive(Clone)]
pub struct CubeMapImportSettings<'m, T: Texel> {
    pub sampling: SamplerSettings,
    pub mode: TextureMode,
    pub usage: TextureUsage,
    pub unwrap: CubeMapUnwrapMode,
    pub mipmaps: TextureMipMaps<'m, 'm, T>,
}

impl<T: Texel> Default for CubeMapImportSettings<'_, T> {
    fn default() -> Self {
        Self {
            sampling: SamplerSettings::default(),
            mode: TextureMode::default(),
            usage: TextureUsage::default(),
            unwrap: CubeMapUnwrapMode::default(),
            mipmaps: TextureMipMaps::Manual { mips: &[] },
        }
    }
}

impl<T: ImageTexel> Asset for CubeMap<T> {
    type Context<'ctx> = Graphics;
    type Settings<'stg> = CubeMapImportSettings<'stg, T>;
    type Err = TextureAssetLoadError;

    fn extensions() -> &'static [&'static str] {
        &["png", "jpg", "jpeg", ".hdr"]
    }

    fn deserialize<'c, 's>(
        data: assets::Data,
        graphics: Self::Context<'c>,
        settings: Self::Settings<'s>,
    ) -> Result<Self, Self::Err> {
        todo!()
    }
}