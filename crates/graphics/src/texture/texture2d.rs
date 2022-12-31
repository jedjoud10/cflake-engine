use std::{marker::PhantomData, mem::ManuallyDrop};

use vulkan::{vk, Allocation};

use crate::{Texel, TextureMode, TextureUsage, Graphics, Texture};

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

    fn dimensionality(&self) -> usize {
        2
    }

    fn layers(&self) -> usize {
        1
    }

    fn usage(&self) -> TextureUsage {
        self.usage
    }
}