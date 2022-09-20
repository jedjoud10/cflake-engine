use super::{Extent, Region, Texel, Texture};
use crate::prelude::TextureMode;
use std::{
    cell::{Cell},
    num::NonZeroU8,
    rc::Rc,
};

// A mip map descriptor contains the two bitfields that contain the read/write flags of each mipmap level
pub struct MipMapDescriptor {
    pub(super) levels: NonZeroU8,
    pub(super) read: Rc<Cell<u64>>,
    pub(super) write: Rc<Cell<u64>>,
}

// This will read from an Rc<Cell<u64>> and check if the bit at the specified location is set
pub(super) fn get_bit(rc: &Rc<Cell<u64>>, location: u8) -> bool {
    let current = rc.get();
    current >> location & 1 == 1
}

// This will write to an Rc<Cell<u64>> and set a specific bit to the specified location
pub(super) fn set_bit(rc: &Rc<Cell<u64>>, location: u8, value: bool) {
    let mut current = rc.get();

    if value {
        current |= 1 << location;
    } else {
        current &= !(1 << location);
    }

    rc.set(current);
}

// Fetch the dimensions of one single mip level that is part of a texture
fn mip_level_extent<T: Texture>(texture: &T, level: u8) -> <<T as Texture>::Region as Region>::E {
    unsafe { <<T::Region as Region>::E as Extent>::get_level_extent(texture.name(), level) }
}

// An immutable mip level that we can use to read from the texture
pub struct MipLevelRef<'a, T: Texture> {
    texture: &'a T,
    level: u8,
    read: Rc<Cell<u64>>,
}

impl<'a, T: Texture> MipLevelRef<'a, T> {
    // Create an immutable mip level
    pub fn new(texture: &'a T, level: u8, read: Rc<Cell<u64>>) -> Self {
        Self {
            texture,
            level,
            read,
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
        unsafe {
            <<T::Region as Region>::E as Extent>::get_level_extent(self.texture.name(), self.level)
        }
    }

    // Get the mip level's region
    pub fn region(&self) -> T::Region {
        T::Region::with_extent(self.dimensions())
    }

    // Download a sub-region of the mip-level, without checking for safety
    pub unsafe fn download_subregion_unchecked(
        &self,
        region: T::Region,
        data: *mut <T::T as Texel>::Storage,
        texels: u32,
    ) {
        T::read_subregion(self.texture.name(), region, self.level, data, texels);
    }

    // Download the whole mip level, without checking for safety
    pub unsafe fn download_unchecked(&self, data: *mut <T::T as Texel>::Storage, texels: u32) {
        T::read(self.texture.name(), self.level, data, texels);
    }

    // Read the texels from a sub-region in the level
    pub fn download_subregion(&self, region: T::Region) -> Vec<<T::T as Texel>::Storage> {
        assert_ne!(region.area(), 0, "Input data length cannot be zero");

        let mut vec = Vec::<<T::T as Texel>::Storage>::with_capacity(region.area() as usize);
        unsafe {
            self.download_subregion_unchecked(region, vec.as_mut_ptr(), region.area());
        }
        vec
    }

    // Read the texels from the whole level
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

// A mutable mip level that we can use to write to the texture
pub struct MipLevelMut<'a, T: Texture> {
    texture: &'a T,
    level: u8,
    read: Rc<Cell<u64>>,
    write: Rc<Cell<u64>>,
}

impl<'a, T: Texture> MipLevelMut<'a, T> {
    // Create an mutable mip level
    pub fn new(texture: &'a T, level: u8, read: Rc<Cell<u64>>, write: Rc<Cell<u64>>) -> Self {
        Self {
            texture,
            level,
            read,
            write,

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
        unsafe {
            <<T::Region as Region>::E as Extent>::get_level_extent(self.texture.name(), self.level)
        }
    }

    // Get the mip level's region
    pub fn region(&self) -> T::Region {
        T::Region::with_extent(self.dimensions())
    }

    // Download a sub-region of the mip-level, without checking for safety
    pub unsafe fn download_subregion_unchecked(
        &self,
        region: T::Region,
        data: *mut <T::T as Texel>::Storage,
        texels: u32,
    ) {
        T::read_subregion(self.texture.name(), region, self.level, data, texels);
    }

    // Download the whole mip level, without checking for safety
    pub unsafe fn download_unchecked(&self, data: *mut <T::T as Texel>::Storage, texels: u32) {
        T::read(self.texture.name(), self.level, data, texels);
    }

    // Read the texels from a sub-region in the level
    pub fn download_subregion(&self, region: T::Region) -> Vec<<T::T as Texel>::Storage> {
        assert_ne!(region.area(), 0, "Input data length cannot be zero");

        assert!(
            self.texture.is_region_valid(region),
            "Access region is invalid due to size of offset"
        );

        let mut vec = Vec::<<T::T as Texel>::Storage>::with_capacity(region.area() as usize);
        unsafe {
            self.download_subregion_unchecked(region, vec.as_mut_ptr(), region.area());
        }
        vec
    }

    // Read the texels from the whole level
    pub fn download(&self) -> Vec<<T::T as Texel>::Storage> {
        let texels = self.texture.region().area();
        let mut vec = Vec::<<T::T as Texel>::Storage>::with_capacity(texels as usize);
        unsafe {
            self.download_unchecked(vec.as_mut_ptr(), texels);
        }
        vec
    }

    // Update a sub-region of the mip-level, but without checking for safety
    pub unsafe fn upload_subregion_unhecked(
        &mut self,
        region: T::Region,
        data: *const <T::T as Texel>::Storage,
    ) {
        T::update_subregion(self.texture.name(), region, self.level, data)
    }

    // Update the whole mip-level, but without checking for safety
    pub unsafe fn upload_unchecked(&mut self, data: *const <T::T as Texel>::Storage) {
        T::update_subregion(self.texture.name(), self.texture.region(), self.level, data)
    }

    // Update a sub-region of the mip-level using a data slice
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
        assert!(
            self.texture.is_region_valid(region),
            "Access region is invalid due to size of offset"
        );

        unsafe {
            self.upload_subregion_unhecked(region, data.as_ptr());
        }
    }

    // Update the whole mip-level using a data slice
    pub fn upload(&mut self, data: &[<T::T as Texel>::Storage]) {
        assert!(
            (data.len() as u32) == self.texture.region().area(),
            "Input data length is not equal to mip level area surface"
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

    // Set the contents of a sub-region of the texture level to the given value without checking for safety
    pub unsafe fn splat_subregion_unchecked(
        &mut self,
        region: T::Region,
        data: *const <T::T as Texel>::Storage,
    ) {
        T::splat_subregion(self.texture.name(), region, self.level, data)
    }

    // Set the whole contents of the texture level to the specified value without checking for safety
    pub unsafe fn splat_unchecked(&mut self, data: *const <T::T as Texel>::Storage) {
        T::splat(self.texture.name(), self.level, data)
    }

    // Set the contents of a sub-region of the texture level to the given value
    pub fn splat_subregion(&mut self, region: T::Region, val: <T::T as Texel>::Storage) {
        assert_ne!(
            self.texture.mode(),
            TextureMode::Static,
            "Cannot write data to static textures"
        );

        assert!(
            self.texture.is_region_valid(region),
            "Access region is invalid due to size of offset"
        );

        unsafe {
            self.splat_subregion_unchecked(region, &val);
        }
    }

    // Set the whole contents of the texture level to the specified value
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

    // Copy a sub-region from another texture level into this texture without checking for safety
    pub unsafe fn copy_subregion_from_unchecked(&mut self, other: &MipLevelRef<T>, read_region: T::Region, write_offset: <T::Region as Region>::O) {
        T::copy_subregion_from(self.texture.name(), other.texture.name(), self.level, other.level, read_region, write_offset);
    }

    // Copy a whole another texture level into this one without checking for safety
    pub unsafe fn copy_from_unchecked(&mut self, other: &MipLevelRef<T>) {
        let offset = <T::Region as Region>::unit().origin();
        let read_region = <T::Region as Region>::with_extent(self.dimensions());
        T::copy_subregion_from(self.texture.name(), other.texture.name(), self.level, other.level, read_region, offset);
    }

    // Copy a sub-region from another texture level into this texture
    pub fn copy_subregion_from(&mut self, other: &MipLevelRef<T>, read_region: T::Region, write_offset: <T::Region as Region>::O) {
        assert_ne!(
            self.texture.mode(),
            TextureMode::Static,
            "Cannot write data to static textures"
        );

        assert!(
            other.texture.is_region_valid(read_region),
            "Access read region is invalid due to size or offset"
        );

        let reg = <T::Region as Region>::from_raw_parts(write_offset, read_region.extent());
        assert!(
            self.texture.is_region_valid(reg),
            "Access write region is invalid due to size or offset"
        );

        unsafe {
            self.copy_subregion_from_unchecked(other, read_region, write_offset);
        }
    }

    // Copy a whole another texture level into this one
    pub fn copy_from(&mut self, other: &MipLevelRef<T>) {
        assert_ne!(
            self.texture.mode(),
            TextureMode::Static,
            "Cannot write data to static textures"
        );

        assert!(self.dimensions() == other.dimensions(), "Cannot copy from differently sized mip level");

        self.copy_subregion_from(other, <T::Region as Region>::with_extent(self.dimensions()), <T::Region as Region>::unit().origin())
    }
}

impl<'a, T: Texture> Drop for MipLevelMut<'a, T> {
    fn drop(&mut self) {
        set_bit(&self.read, self.level, false);
        set_bit(&self.write, self.level, false);
    }
}
