use std::num::NonZeroU8;

use super::{Region, Texture};
use crate::{
    Extent, TextureAsTargetError, TextureSamplerError, RenderTarget, Texel,
};

// This enum tells the texture how exactly it should create it's mipmaps
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureMipMaps<'mip, 'map, T: Texel> {
    // Disable mipmap generation for the texture
    Disabled,

    // Automatic mipmap generation based on the texture dimensions
    #[default]
    Automatic,

    // Clamped automatic mipmap generation (to limit number of mips)
    // If levels is less than 2, then mipmapping will be disabled
    // Will be clamped to the maximum number of levels possible
    AutomaticClamped {
        max: NonZeroU8,
    },

    // Manual mip map generation with the specified texels at each mip level
    // Will be clamped to the maximum number of levels possible
    Manual {
        mips: &'map [&'mip [T::Storage]],
    },
}

// TODO: Figure out how to store and create vk::Views for each mipmap
// Should they be stored in a SmallArray or SmallVec??

// An immutable mip level that we can use to read from the texture
pub struct MipLevelRef<'a, T: Texture> {
    texture: &'a T,
    level: u8,
}

// Helper methods
impl<'a, T: Texture> MipLevelRef<'a, T> {
    // Creat a mip level reference from it's raw parts
    pub unsafe fn from_raw_parts(texture: &'a T, level: u8) -> Self {
        Self { texture, level }
    }

    // Get the underlying texture
    pub fn texture(&self) -> &T {
        self.texture
    }

    // Get the mip level of the current level
    pub fn level(&self) -> u8 {
        self.level
    }

    // Get the view for this mip
    pub fn view(&self) -> &wgpu::TextureView {
        &self.texture().views()[self.level as usize]
    }

    // Get the mip level's dimensions
    pub fn dimensions(&self) -> <T::Region as Region>::E {
        self.texture.dimensions().mip_level_dimensions(self.level)
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
    level: u8,
}

// Helper methods
impl<'a, T: Texture> MipLevelMut<'a, T> {
    // Creat a mip level mutable reference from it's raw parts
    pub unsafe fn from_raw_parts(texture: &'a T, level: u8) -> Self {
        Self { texture, level }
    }

    // Get the underlying texture
    pub fn texture(&self) -> &T {
        self.texture
    }

    // Get the mip level of the current level
    pub fn level(&self) -> u8 {
        self.level
    }

    // Get the view for this mip
    pub fn view(&self) -> &wgpu::TextureView {
        &self.texture().views()[self.level as usize]
    }

    // Get the mip level's dimensions
    pub fn dimensions(&self) -> <T::Region as Region>::E {
        self.texture.dimensions().mip_level_dimensions(self.level)
    }

    // Get the mip level's region
    pub fn region(&self) -> T::Region {
        T::Region::with_extent(self.dimensions())
    }

    // Try to get a render target so we can render to this one mip level
    pub fn as_render_target(
        &mut self,
    ) -> Result<RenderTarget<T::T>, TextureAsTargetError> {
        todo!()
    }
}

// Implementation of unsafe methods
impl<'a, T: Texture> MipLevelMut<'a, T> {}

// Implementation of safe methods
impl<'a, T: Texture> MipLevelMut<'a, T> {}
