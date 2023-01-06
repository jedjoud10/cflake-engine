use vulkan::vk;
use crate::{TextureSamplerError, Sampler, Extent, RenderTarget, TextureAsTargetError};
use super::{Region, Texture};

// TODO: Figure out how to store and create vk::Views for each mipmap
// Should they be stored in a SmallArray or SmallVec??

// An immutable mip level that we can use to read from the texture
pub struct MipLevelRef<'a, T: Texture> {
    texture: &'a T,
    view: vk::ImageView,
    level: u8,
}

// Helper methods
impl<'a, T: Texture> MipLevelRef<'a, T> {
    // Creat a mip level reference from it's raw parts
    pub unsafe fn from_raw_parts(
        texture: &'a T,
        view: vk::ImageView,
        level: u8
    ) -> Self {
        Self {
            texture,
            view,
            level,
        }
    }

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
        self.texture.dimensions().mip_level_dimensions(self.level)
    }

    // Get the mip level's region
    pub fn region(&self) -> T::Region {
        T::Region::with_extent(self.dimensions())
    }

    // Try to get a sampler for this one mip level
    pub fn as_sampler(&self) -> Result<Sampler<T>, TextureSamplerError> {
        todo!()
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
    // Creat a mip level mutable reference from it's raw parts
    pub unsafe fn from_raw_parts(
        texture: &'a T,
        view: vk::ImageView,
        level: u8
    ) -> Self {
        Self {
            texture,
            view,
            level,
        }
    }

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
        self.texture.dimensions().mip_level_dimensions(self.level)
    }

    // Get the mip level's region
    pub fn region(&self) -> T::Region {
        T::Region::with_extent(self.dimensions())
    }

    // Try to get a sampler for this one mip level
    pub fn as_sampler(&self) -> Result<Sampler<T>, TextureSamplerError> {
        todo!()
    }

    // Try to get a render target so we can render to this one mip level
    pub fn as_target(&mut self) -> Result<RenderTarget<T::T>, TextureAsTargetError> {
        Ok(unsafe { RenderTarget::from_raw_parts(self.texture.image(), self.view) })
    }
}

// Implementation of unsafe methods
impl<'a, T: Texture> MipLevelMut<'a, T> {}

// Implementation of safe methods
impl<'a, T: Texture> MipLevelMut<'a, T> {}
