use std::{marker::PhantomData, mem::ManuallyDrop, time::Instant};

use assets::Asset;

use crate::{
    Graphics, ImageTexel, Texel, Texture, TextureAssetLoadError,
    TextureMode, TextureUsage,
};

// A 2D texture that contains multiple texels that have their own channels
// Each texel can be either a single value, RG, RGB, or even RGBA
pub struct Texture2D<T: Texel> {
    // Raw WGPU
    texture: wgpu::Texture,
    view: wgpu::TextureView,

    // Main texture settings
    dimensions: vek::Extent2<u32>,

    // Permissions
    usage: TextureUsage,
    mode: TextureMode,
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

    fn mode(&self) -> TextureMode {
        self.mode
    }

    fn usage(&self) -> TextureUsage {
        self.usage
    }

    fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    unsafe fn from_raw_parts(
        graphics: &Graphics,
        texture: wgpu::Texture,
        view: wgpu::TextureView,
        dimensions: <Self::Region as crate::Region>::E,
        usage: TextureUsage,
        mode: TextureMode,
    ) -> Self {
        Self {
            texture,
            view,
            dimensions,
            usage,
            mode,
            _phantom: PhantomData,
            graphics: graphics.clone(),
        }
    }
}

impl<T: ImageTexel> Asset for Texture2D<T> {
    type Context<'ctx> = &'ctx Graphics;
    type Settings<'stg> = ();
    type Err = TextureAssetLoadError;

    fn extensions() -> &'static [&'static str] {
        &["png", "jpg", "jpeg"]
    }

    fn deserialize<'c, 's>(
        data: assets::Data,
        graphics: Self::Context<'c>,
        _settings: Self::Settings<'s>,
    ) -> Result<Self, Self::Err> {
        let i = Instant::now();
        let image = image::load_from_memory(data.bytes())
            .map_err(TextureAssetLoadError::Deserialization)?;
        log::debug!(
            "Took {:?} to deserialize texture {:?}",
            i.elapsed(),
            data.path()
        );

        let dimensions =
            vek::Extent2::new(image.width(), image.height());
        let texels = T::to_image_texels(image);

        Self::from_texels(
            graphics,
            Some(&texels),
            dimensions,
            TextureMode::Dynamic,
            TextureUsage::Placeholder,
        )
        .map_err(TextureAssetLoadError::Initialization)
    }
}
