use std::{cell::{RefCell, Cell}, rc::Rc};

use crate::prelude::TextureMode;
use super::{Extent, Region, Texel, Texture};

// This wrapper contains all the mipmaps from the texture
// We can use this accessor to fetch multiple mutable mip maps at the same time as well
pub struct MipMap<'a, T: Texture> {
    pub(super) texture: &'a mut T,
    pub(super) read: Rc<Cell<u64>>,
    pub(super) write: Rc<Cell<u64>>,
}

// This will read from an Rc<Cell<u64>> and check if the bit at the specified location is set
fn get_bit(rc: &Rc<Cell<u64>>, location: u8) -> bool {
    let current = rc.get();
    current >> location & 1 == 1
}

// This will write to an Rc<Cell<u64>> and set a specific bit to the specified location
fn set_bit(rc: &Rc<Cell<u64>>, location: u8, value: bool) {
    let mut current = rc.get();

    if value {
        current |= 1 << location;
    } else {
        current &= !(1 << location);
    }

    rc.set(current);
} 

impl<'a, T: Texture> MipMap<'a, T> {
    // Get a single mip level from the texture, immutably
    // This will fail if the mip level is currently being used mutably
    pub fn mip(&self, level: u8) -> Option<MipLevelRef<T>> {
        if level > self.texture.levels().get() {
            return None;
        }

        if get_bit(&self.write, level) {
            return None;
        }

        set_bit(&self.read, level, true);
        
        Some(MipLevelRef {
            texture: self.texture,
            read: self.read.clone(),
            level,
        })
    }

    // Get a single mip level from the texture, mutably
    // This will fail if the mip level is currently being used mutably or being read from
    pub fn mip_mut(&mut self, level: u8) -> Option<MipLevelMut<T>> {
        if level > self.texture.levels().get() {
            return None;
        }

        if get_bit(&self.write, level) || get_bit(&self.read, level) {
            return None;
        }

        set_bit(&self.read, level, true);
        set_bit(&self.write, level, true);
        
        Some(MipLevelMut {
            texture: self.texture,
            read: self.read.clone(),
            write: self.write.clone(),
            level,
        })
    }
}

// An immutable mip level that we can use to read from the texture
pub struct MipLevelRef<'a, T: Texture> {
    texture: &'a T,
    level: u8,
    read: Rc<Cell<u64>>,
}

impl<'a, T: Texture> MipLevelRef<'a, T> {
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
            <<T::Region as Region>::E as Extent>::get_level_extent(self.texture.name(), self.level)
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

impl<'a, T: Texture> Drop for MipLevelRef<'a, T> {
    fn drop(&mut self) {
        set_bit(&self.read, self.level, false);
    }
}

// A mutable mip layer that we can use to write to the texture
pub struct MipLevelMut<'a, T: Texture> {
    texture: &'a T,
    level: u8,
    read: Rc<Cell<u64>>,
    write: Rc<Cell<u64>>,
}

impl<'a, T: Texture> MipLevelMut<'a, T> {
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
            <<T::Region as Region>::E as Extent>::get_level_extent(self.texture.name(), self.level)
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

impl<'a, T: Texture> Drop for MipLevelMut<'a, T> {
    fn drop(&mut self) {
        set_bit(&self.read, self.level, false);
        set_bit(&self.write, self.level, false);
    }
}