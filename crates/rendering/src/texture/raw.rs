use std::ffi::c_void;

use super::{Extent, Region, TexelLayout, Texture};

// A raw texture allocator that will simply call the unique OpenGL functions for each texture type
pub trait TextureAllocator {
    // A texture region that might cover the whole texture or just partially
    type TexelRegion: Region;

    // Allocate some immutable texture storage during texture initialization
    unsafe fn alloc_immutable_storage(
        name: u32,
        extent: <Self::TexelRegion as Region>::E,
        levels: u8,
        ptr: *const c_void,
    );

    // Allocate some mutable(resizable) texture during texture initialization
    // PS: This will allocate the texture storage for only one level
    unsafe fn alloc_resizable_storage(
        name: u32,
        extent: <Self::TexelRegion as Region>::E,
        unique_level: u8,
        ptr: *const c_void,
    );

    // Update a sub-region of the raw texture
    unsafe fn update_subregion(name: u32, region: Self::TexelRegion, ptr: *const c_void);
}
