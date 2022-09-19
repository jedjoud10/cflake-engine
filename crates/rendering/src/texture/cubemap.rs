use assets::Asset;

use super::{
    ImageTexel, MipMapDescriptor, Region, Texel, Texture, TextureImportSettings, TextureMode,
};
use crate::context::{Context, ToGlName, ToGlTarget};
use std::{ffi::c_void, marker::PhantomData, ptr::null};


// A cubemap texture that contains 6 different faces that each contain a square texture2D
// Cubemap textures are mostly used for environment mapping and reflections
// Cubemaps are internally stored in this data order:
// PositiveX, NegativeX, PositiveY, NegativeY, PositiveZ, NegativeZ
pub struct CubeMap2D<T: Texel> {
    // Internal OpenGL shit
    name: u32,

    // Main texture settings
    dimensions: vek::Extent2<u16>,
    mode: TextureMode,
    mipmap: MipMapDescriptor,

    // Boo (also sets Texture2D as !Sync and !Send)
    _phantom: PhantomData<*const T>,
}

impl<T: Texel> ToGlName for CubeMap2D<T> {
    fn name(&self) -> u32 {
        self.name
    }
}

impl<T: Texel> ToGlTarget for CubeMap2D<T> {
    fn target() -> u32 {
        gl::TEXTURE_CUBE_MAP
    }
}

impl<T: Texel> Texture for CubeMap2D<T> {
    type Region = (vek::Vec2<u16>, vek::Extent2<u16>);
    type T = T;

    fn dimensions(&self) -> <Self::Region as super::Region>::E {
        self.dimensions
    }

    fn mode(&self) -> super::TextureMode {
        self.mode
    }

    fn mipmap_descriptor(&self) -> &MipMapDescriptor {
        &self.mipmap
    }

    unsafe fn from_raw_parts(
        name: u32,
        dimensions: <Self::Region as super::Region>::E,
        mode: TextureMode,
        mipmap: MipMapDescriptor,
    ) -> Self {
        Self {
            name,
            dimensions,
            mode,
            mipmap,
            _phantom: Default::default(),
        }
    }

    unsafe fn alloc_immutable_storage(
        name: u32,
        extent: <Self::Region as Region>::E,
        levels: u8,
        ptr: *const <Self::T as Texel>::Storage,
    ) {
    }

    unsafe fn alloc_resizable_storage(
        name: u32,
        extent: <Self::Region as Region>::E,
        unique_level: u8,
        ptr: *const <Self::T as Texel>::Storage,
    ) {        
        let extent = extent.as_::<i32>();
    }

    unsafe fn update_subregion(
        name: u32,
        region: Self::Region,
        level: u8,
        ptr: *const <Self::T as Texel>::Storage,
    ) {
        let origin = region.origin().as_::<i32>();
        let extent = region.extent().as_::<i32>();
    }

    unsafe fn splat_subregion(
        name: u32,
        region: Self::Region,
        level: u8,
        ptr: *const <Self::T as Texel>::Storage,
    ) {
        let origin = region.origin().as_::<i32>();
        let extent = region.extent().as_::<i32>();
    }

    unsafe fn splat(name: u32, level: u8, ptr: *const <Self::T as Texel>::Storage) {
        gl::ClearTexImage(name, level as i32, T::FORMAT, T::TYPE, ptr as *const c_void);
    }

    unsafe fn read_subregion(
        name: u32,
        region: Self::Region,
        level: u8,
        ptr: *mut <Self::T as Texel>::Storage,
        texels: u32,
    ) {
        let origin = region.origin().as_::<i32>();
        let extent = region.extent().as_::<i32>();
        let size = texels as u32 * T::bytes();
    }

    unsafe fn read(name: u32, level: u8, ptr: *mut <Self::T as Texel>::Storage, texels: u32) {
        let size = texels as u32 * T::bytes();
    }

    unsafe fn copy_subregion_from(name: u32, other_name: u32, level: u8, other_level: u8, region: Self::Region, offset: <Self::Region as Region>::O) {
        let origin = region.origin().as_::<i32>();
        let extent = region.extent().as_::<i32>();
        let offset = offset.as_::<i32>();
    }
}