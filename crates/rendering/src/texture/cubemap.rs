use assets::Asset;

use super::{
    ImageTexel, MipMapDescriptor, Region, Texel, Texture, TextureImportSettings, TextureMode, Extent,
};
use crate::context::{Context, ToGlName, ToGlTarget};
use std::{ffi::c_void, marker::PhantomData, ptr::null, mem::size_of};


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
    type Region = (vek::Vec3<u16>, vek::Extent2<u16>);
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

    fn is_region_valid(&self, region: Self::Region) -> bool {
        let extent = <Self::Region as Region>::extent_from_origin(region.origin()) + region.extent();
        let dimensions = extent.is_self_smaller(self.dimensions());
        dimensions && region.origin().z < 6
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
        let extent = extent.as_::<i32>();
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, name);
        gl::TextureStorage2D(
            name,
            levels as i32,
            T::INTERNAL_FORMAT,
            extent.w,
            extent.h,
        );

        if ptr != null() {
            for face in 0..6u32 {
                let offset = face as usize * extent.product() as usize;
                let offsetted_ptr = ptr.offset(offset as isize);

                gl::TexSubImage2D(
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + face,
                    0,
                    0,
                    0,
                    extent.w,
                    extent.h,
                    T::FORMAT,
                    T::TYPE,
                    offsetted_ptr as *const c_void,
                );
            }            
        }
    }

    unsafe fn alloc_resizable_storage(
        name: u32,
        extent: <Self::Region as Region>::E,
        unique_level: u8,
        ptr: *const <Self::T as Texel>::Storage,
    ) {        
        let extent = extent.as_::<i32>();
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, name);

        for face in 0..6u32 {
            let offset = face as usize * extent.product() as usize;
            let offsetted_ptr = ptr.offset(offset as isize);

            gl::TexImage2D(
                gl::TEXTURE_CUBE_MAP_POSITIVE_X + face,
                unique_level as i32,
                T::INTERNAL_FORMAT as i32,
                extent.w,
                extent.h,
                0,
                T::FORMAT,
                T::TYPE,
                offsetted_ptr as *const c_void,
            );
        }        
    }

    unsafe fn update_subregion(
        name: u32,
        region: Self::Region,
        level: u8,
        ptr: *const <Self::T as Texel>::Storage,
    ) {
        let origin = region.origin().as_::<i32>();
        let extent = region.extent().as_::<i32>();
        let face = origin.z as u32;
        gl::BindTexture(gl::TEXTURE_CUBE_MAP, name);
        gl::TexSubImage2D(
            gl::TEXTURE_CUBE_MAP_POSITIVE_X + face,
            level as i32,
            origin.x,
            origin.y,
            extent.w,
            extent.h,
            T::FORMAT,
            T::TYPE,
            ptr as *const c_void,
        );
    }

    unsafe fn splat_subregion(
        name: u32,
        region: Self::Region,
        level: u8,
        ptr: *const <Self::T as Texel>::Storage,
    ) {
        let origin = region.origin().as_::<i32>();
        let extent = region.extent().as_::<i32>();
        gl::ClearTexSubImage(
            name,
            level as i32,
            origin.x,
            origin.y,
            origin.z,
            extent.w,
            extent.h,
            1,
            T::FORMAT,
            T::TYPE,
            ptr as *const c_void,
        );       
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
        gl::GetTextureSubImage(
            name,
            level as i32,
            origin.x,
            origin.y,
            origin.z,
            extent.w,
            extent.h,
            1,
            T::FORMAT,
            T::TYPE,
            size as i32,
            ptr as *mut c_void,
        );
    }

    unsafe fn read(name: u32, level: u8, ptr: *mut <Self::T as Texel>::Storage, texels: u32) {
        let size = texels as u32 * T::bytes();
        gl::GetTextureImage(
            name,
            level as i32,
            T::FORMAT,
            T::TYPE,
            size as i32,
            ptr as *mut c_void,
        )
    }

    unsafe fn copy_subregion_from(name: u32, other_name: u32, level: u8, other_level: u8, region: Self::Region, offset: <Self::Region as Region>::O) {
        let origin = region.origin().as_::<i32>();
        let extent = region.extent().as_::<i32>();
        let offset = offset.as_::<i32>();

        gl::CopyImageSubData(
            other_name,
            gl::TEXTURE_CUBE_MAP,
            other_level as i32,
            origin.x,
            origin.y,
            origin.z,
            name,
            gl::TEXTURE_CUBE_MAP,
            level as i32,
            offset.x,
            offset.y,
            offset.z,
            extent.w,
            extent.h,
            1,
        );
    }
}