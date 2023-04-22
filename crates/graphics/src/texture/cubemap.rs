use std::{marker::PhantomData, mem::ManuallyDrop, sync::Arc, time::Instant};

use assets::Asset;
use smallvec::SmallVec;

use crate::{
    Extent, Graphics, ImageTexel, Sampler, SamplerSettings, Texel, Texture, TextureAssetLoadError,
    TextureInitializationError, TextureMipMaps, TextureMode, TextureUsage,
};

// A cubemap texture that contains multiple texels that have their own channels
// A cubemap contains 6 base layer, that represent each side of the cube
// TODO: TEST
pub struct CubeMap<T: Texel> {
    // Raw WGPU
    texture: wgpu::Texture,
    views: Option<Vec<wgpu::TextureView>>,

    // Main texture settings
    dimensions: vek::Extent2<u32>,

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
        views: Option<Vec<wgpu::TextureView>>,
        dimensions: <Self::Region as crate::Region>::E,
    ) {
        self.texture = texture;
        self.views = views;
        self.dimensions = dimensions;
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
