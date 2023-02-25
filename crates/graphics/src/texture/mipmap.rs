use std::num::NonZeroU8;

use super::{Region, Texture};
use crate::{
    Extent, RenderTarget, Texel, TextureAsTargetError,
    TextureSamplerError, ColorTexel,
};

// This enum tells the texture how exactly it should create it's mipmaps
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TextureMipMaps<'mip, 'map, T: Texel> {
    // Disable mipmap generation for the texture
    Disabled,

    // Manual mip level generation based on the texture dimensions
    // This will keep the mip levels uninitialized however
    Zeroed {
        // Clamped automatic mipmap generation (to limit number of mips)
        // If levels is less than 2, then mipmapping will be disabled
        // Will be clamped to the maximum number of levels possible
        clamp: Option<NonZeroU8>,
    },

    // Manual mip map generation with the specified texels at each mip level
    // Will be clamped to the maximum number of levels possible

    // NOTE: If this is of length 0 and the texture is loaded from a file, then the 
    // mips will be generated automatically based on image file contents
    Manual {
        mips: &'map [&'mip [T::Storage]],
    },
}

impl<T: Texel> Default for TextureMipMaps<'_, '_, T> {
    fn default() -> Self {
        Self::Disabled
    }
}

// Calculate mip levels based on the given color data and size
// Returns None if the texture isn't a power of two texture
pub fn generate_mip_map<T: ColorTexel, E: Extent>(
    base: &[T::Storage],
    extent: E,
) -> Option<Vec<Vec<T::Storage>>> {
    // Create manual mip maps for this texture
    let dimension = <E as Extent>::dimensionality();
    let name = utils::pretty_type_name::<T>();
    let levels = extent.levels()?.get() as u32;
    log::debug!("Creating mip-data (max = {levels})for imported texture {dimension:?}, <{name}>");

    // Iterate over the levels and fill them up
    // (like how ceddy weddy fills me up inside >.<) 
    for i in 1..levels {
        let downscaled = extent.mip_level_dimensions(i as u8); 
        log::debug!(
            "Create mipdata for layer <{i}> from imported image, {}x{}x{}",
            downscaled.width(),
            downscaled.height(),
            downscaled.depth()
        );
    }

    todo!()
}

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

impl<'a, T: Texture> MipLevelRef<'a, T> {
    // Read some pixels from the mip level region to the given destination
    fn read(
        &self,
        dst: &mut [<T::T as Texel>::Storage],
        subregion: Option<T::Region>, 
    ) {
        todo!()
    } 
}

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

impl<'a, T: Texture> MipLevelMut<'a, T> {
    // Read some pixels from the mip level region to the given destination
    fn read(
        &self,
        dst: &mut [<T::T as Texel>::Storage],
        subregion: Option<T::Region>, 
    ) {
        todo!()
    } 

    // Write some pixels to the mip level region from the given source
    fn write(
        &mut self,
        src: &[<T::T as Texel>::Storage],
        subregion: Option<T::Region>, 
    ) {
        todo!()
    } 

    // Copy a sub-region from another level into this level
    pub fn copy_subregion_from(
        &mut self,
        other: &MipLevelRef<T>,
        src_subregion: Option<T::Region>,
        dst_subregion: Option<T::Region>
    ) {
    }
}
