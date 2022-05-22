use std::{
    num::{NonZeroU32, NonZeroU8},
    rc::Rc,
};

use super::{Bindless, Dim, TexelLayout, Texture, TextureMode};

// An allocator is the bare-bones wrapper around the raw OpenGL functions
pub unsafe trait TextureAllocator: Texture {
    // This will allocate som immutable storage for this texture
    unsafe fn alloc_immutable_storage(name: NonZeroU32, levels: u8, dimensions: Self::Dimensions);

    // Allocate a mutable storage (using glImage*)
    unsafe fn alloc_resizable_storage(name: NonZeroU32, level: u8, dimensions: Self::Dimensions, ptr: *const Self::Layout);

    // Update a sub-region of the texture
    unsafe fn update_sub_region(name: NonZeroU32, level: u8, region: Self::Region, ptr: *const Self::Layout);

    // Create a new texture object using the raw texture parts
    unsafe fn from_raw_parts(name: NonZeroU32, dimensions: Self::Dimensions, mode: TextureMode, levels: NonZeroU8, bindless: Option<Rc<Bindless>>) -> Self;
}
