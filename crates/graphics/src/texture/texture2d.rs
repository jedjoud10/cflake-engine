use std::{marker::PhantomData, mem::ManuallyDrop, time::Instant};

use assets::Asset;
use vulkan::{vk, Allocation};

use crate::{Texel, TextureMode, TextureUsage, Graphics, Texture, ImageTexel, TextureAssetLoadError};

// A 2D texture that contains multiple texels that have their own channels
// Each texel can be either a single value, RG, RGB, or even RGBA
pub struct Texture2D<T: Texel> {
    // Raw vulkan
    image: vk::Image,
    allocation: ManuallyDrop<Allocation>,
    whole_view: vk::ImageView,

    // Main texture settings
    dimensions: vek::Extent2<u32>,

    // Legal permissions
    usage: TextureUsage,
    mode: TextureMode,

    // Keep the graphics API alive
    graphics: Graphics,
    _phantom: PhantomData<T>,
}

impl<T: Texel> Drop for Texture2D<T> {
    fn drop(&mut self) {
        unsafe {
            let alloc = ManuallyDrop::take(&mut self.allocation);
            self.graphics.device().destroy_image(self.image, alloc);
        }
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

    unsafe fn from_raw_parts(
        image: vk::Image,
        whole_view: vk::ImageView,
        allocation: Allocation,
        dimensions: <Self::Region as crate::Region>::E,
        usage: TextureUsage,
        mode: TextureMode,
        graphics: &Graphics,
    ) -> Self {
        Self {
            image,
            allocation: ManuallyDrop::new(allocation),
            whole_view,
            dimensions,
            usage,
            mode,
            graphics: graphics.clone(),
            _phantom: PhantomData,
        }
    }

    fn allocation(&self) -> &Allocation {
        &self.allocation
    }

    fn allocation_mut(&mut self) -> &mut Allocation {
        &mut self.allocation
    }
}

impl<T: ImageTexel> Asset for Texture2D<T> {
    type Args<'args> = &'args Graphics;
    type Err = TextureAssetLoadError;

    fn extensions() -> &'static [&'static str] {
        &["png", "jpg", "jpeg"]
    }

    fn deserialize<'args>(
        data: assets::Data,
        args: Self::Args<'args>,
    ) -> Result<Self, Self::Err> {
        let i = Instant::now();
        let image = image::load_from_memory(data.bytes())
            .map_err(TextureAssetLoadError::Deserialization)?;
        log::debug!("Took {:?} to deserialize texture {:?}", i.elapsed(), data.path());

        let dimensions = vek::Extent2::new(image.width(), image.height());
        let texels = T::to_image_texels(image);

        Self::from_texels(
            args,
            &texels,
            dimensions,
            TextureMode::Dynamic,
            TextureUsage::Placeholder,
        ).map_err(TextureAssetLoadError::Initialization)
    }
}