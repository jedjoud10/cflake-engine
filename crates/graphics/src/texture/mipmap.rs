use vulkan::vk;

use super::{Region, Texture};

// An immutable mip level that we can use to read from the texture
pub struct MipLevelRef<'a, T: Texture> {
    texture: &'a T,
    view: vk::ImageView,
    level: u8,
}

// Helper methods
impl<'a, T: Texture> MipLevelRef<'a, T> {
    // Get the underlying texture
    pub fn texture(&self) -> &T {
        self.texture
    }

    // Get the mip level of the current level
    pub fn level(&self) -> u8 {
        self.level
    }

    // Get the mip level's dimensions
    pub fn dimensions(&self) -> <T::Region as Region>::E {
        todo!()
    }

    // Get the mip level's region
    pub fn region(&self) -> T::Region {
        T::Region::with_extent(self.dimensions())
    }
}

// Implementation of unsafe methods
impl<'a, T: Texture> MipLevelRef<'a, T> {}

// Implementation of safe methods
impl<'a, T: Texture> MipLevelRef<'a, T> {}

// A mutable mip level that we can use to write to the texture
pub struct MipLevelMut<'a, T: Texture> {
    texture: &'a T,
    view: vk::ImageView,
    level: u8,
}

// Helper methods
impl<'a, T: Texture> MipLevelMut<'a, T> {
    // Get the underlying texture
    pub fn texture(&self) -> &T {
        self.texture
    }

    // Get the mip level of the current level
    pub fn level(&self) -> u8 {
        self.level
    }

    // Get the mip level's dimensions
    pub fn dimensions(&self) -> <T::Region as Region>::E {
        todo!()
    }

    // Get the mip level's region
    pub fn region(&self) -> T::Region {
        T::Region::with_extent(self.dimensions())
    }
}

// Implementation of unsafe methods
impl<'a, T: Texture> MipLevelMut<'a, T> {}

// Implementation of safe methods
impl<'a, T: Texture> MipLevelMut<'a, T> {}
