use crate::prelude::TextureMode;

use super::{Extent, Region, Texel, Texture};

// An immutable mip layer that we can use to read from the texture
pub struct MipLevelRef<'a, T: Texture> {
    texture: &'a T,
    level: u8,
}

impl<'a, T: Texture> MipLevelRef<'a, T> {
    // Create a new mip layer view using a texture and a level
    pub(super) fn new(texture: &'a T, level: u8) -> Self {
        Self { texture, level }
    }

    // Get the underlying texture
    pub fn texture(&self) -> &T {
        self.texture
    }

    // Get the mip level of the current layer
    pub fn level(&self) -> u8 {
        self.level
    }

    // Get the mip layer's dimensions
    pub fn dimensions(&self) -> <T::Region as Region>::E {
        unsafe {
            <<T::Region as Region>::E as Extent>::get_layer_extent(self.texture.name(), self.level)
        }
    }

    // Download a sub-region of the mip-layer, without checking for safety
    pub unsafe fn download_subregion_unchecked(
        &self,
        region: T::Region,
        data: *mut <T::T as Texel>::Storage,
        texels: u32,
    ) {
        T::read_subregion(self.texture.name(), region, self.level, data, texels);
    }

    // Download the whole mip layer, without checking for safety
    pub unsafe fn download_unchecked(&self, data: *mut <T::T as Texel>::Storage, texels: u32) {
        T::read(self.texture.name(), self.level, data, texels);
    }

    // Read the texels from a sub-region in the layer
    pub fn download_subregion(&self, region: T::Region) -> Vec<<T::T as Texel>::Storage> {
        assert_ne!(region.area(), 0, "Input data length cannot be zero");

        let mut vec = Vec::<<T::T as Texel>::Storage>::with_capacity(region.area() as usize);
        unsafe {
            self.download_subregion_unchecked(region, vec.as_mut_ptr(), region.area());
        }
        vec
    }

    // Read the texels from the whole layer
    pub fn download(&self) -> Vec<<T::T as Texel>::Storage> {
        let texels = self.texture.region().area();
        let mut vec = Vec::<<T::T as Texel>::Storage>::with_capacity(texels as usize);
        unsafe {
            self.download_unchecked(vec.as_mut_ptr(), texels);
        }
        vec
    }
}

// A mutable mip layer that we can use to write to the texture
pub struct MipLevelMut<'a, T: Texture> {
    texture: &'a mut T,
    level: u8,
}

impl<'a, T: Texture> MipLevelMut<'a, T> {
    // Create a new mip layer mutable view using a texture and a level
    pub(super) fn new(texture: &'a mut T, level: u8) -> Self {
        Self { texture, level }
    }

    // Get the underlying texture
    pub fn texture(&self) -> &T {
        self.texture
    }

    // Get the mip level of the current layer
    pub fn level(&self) -> u8 {
        self.level
    }

    // Get the mip layer's dimensions
    pub fn dimensions(&self) -> <T::Region as Region>::E {
        unsafe {
            <<T::Region as Region>::E as Extent>::get_layer_extent(self.texture.name(), self.level)
        }
    }

    // Download a sub-region of the mip-layer, without checking for safety
    pub unsafe fn download_subregion_unchecked(
        &self,
        region: T::Region,
        data: *mut <T::T as Texel>::Storage,
        texels: u32,
    ) {
        T::read_subregion(self.texture.name(), region, self.level, data, texels);
    }

    // Download the whole mip layer, without checking for safety
    pub unsafe fn download_unchecked(&self, data: *mut <T::T as Texel>::Storage, texels: u32) {
        T::read(self.texture.name(), self.level, data, texels);
    }

    // Read the texels from a sub-region in the layer
    pub fn download_subregion(&self, region: T::Region) -> Vec<<T::T as Texel>::Storage> {
        assert_ne!(region.area(), 0, "Input data length cannot be zero");

        let mut vec = Vec::<<T::T as Texel>::Storage>::with_capacity(region.area() as usize);
        unsafe {
            self.download_subregion_unchecked(region, vec.as_mut_ptr(), region.area());
        }
        vec
    }

    // Read the texels from the whole layer
    pub fn download(&self) -> Vec<<T::T as Texel>::Storage> {
        let texels = self.texture.region().area();
        let mut vec = Vec::<<T::T as Texel>::Storage>::with_capacity(texels as usize);
        unsafe {
            self.download_unchecked(vec.as_mut_ptr(), texels);
        }
        vec
    }

    // Update a sub-region of the mip-layer, but without checking for safety
    pub unsafe fn upload_subregion_unhecked(
        &mut self,
        region: T::Region,
        data: *const <T::T as Texel>::Storage,
    ) {
        T::update_subregion(self.texture.name(), region, self.level, data)
    }

    // Update the whole mip-layer, but without checking for safety
    pub unsafe fn upload_unchecked(&mut self, data: *const <T::T as Texel>::Storage) {
        T::update_subregion(self.texture.name(), self.texture.region(), self.level, data)
    }

    // Update a sub-region of the mip-layer using a data slice
    pub fn upload_subregion(&mut self, region: T::Region, data: &[<T::T as Texel>::Storage]) {
        assert!(
            (data.len() as u32) == region.area(),
            "Input data length is not equal to region area surface"
        );
        assert_ne!(data.len(), 0, "Input data length cannot be zero");
        assert_ne!(
            self.texture.mode(),
            TextureMode::Static,
            "Cannot write data to static textures"
        );

        unsafe {
            self.upload_subregion_unhecked(region, data.as_ptr());
        }
    }

    // Update the whole mip-layer using a data slice
    pub fn upload(&mut self, data: &[<T::T as Texel>::Storage]) {
        assert!(
            (data.len() as u32) == self.texture.region().area(),
            "Input data length is not equal to mip layer area surface"
        );
        assert_ne!(
            self.texture.mode(),
            TextureMode::Static,
            "Cannot write data to static textures"
        );

        unsafe {
            self.upload_unchecked(data.as_ptr());
        }
    }

    // Set the contents of a sub-region of the texture layer to the given value without checking for safety
    pub unsafe fn splat_subregion_unchecked(
        &mut self,
        region: T::Region,
        data: *const <T::T as Texel>::Storage,
    ) {
        T::splat_subregion(self.texture.name(), region, self.level, data)
    }

    // Set the whole contents of the texture layer to the specified value without checking for safety
    pub unsafe fn splat_unchecked(&mut self, data: *const <T::T as Texel>::Storage) {
        T::splat(self.texture.name(), self.level, data)
    }

    // Set the contents of a sub-region of the texture layer to the given value
    pub fn splat_subregion(&mut self, region: T::Region, val: <T::T as Texel>::Storage) {
        assert_ne!(
            self.texture.mode(),
            TextureMode::Static,
            "Cannot write data to static textures"
        );

        unsafe {
            self.splat_subregion_unchecked(region, &val);
        }
    }

    // Set the whole contents of the texture layer to the specified value
    pub fn splat(&mut self, val: <T::T as Texel>::Storage) {
        assert_ne!(
            self.texture.mode(),
            TextureMode::Static,
            "Cannot write data to static textures"
        );

        unsafe {
            self.splat_unchecked(&val);
        }
    }
}