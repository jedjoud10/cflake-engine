use super::{MipMapDescriptor, MultiLayerTexture, Region, Texel, Texture, TextureMode};
use crate::context::{ToGlName, ToGlTarget};
use std::{ffi::c_void, marker::PhantomData, ptr::null};

// A 3D texture that contains multiple voxels that have their own channels
// Each voxel can be either a single value, RG, RGB, or even RGBA
// These individual voxel are also called texels, since they are used within the texture
pub struct Texture3D<T: Texel> {
    // Internal OpenGL shit
    name: u32,

    // Main texture settings
    dimensions: vek::Extent3<u16>,
    mode: TextureMode,
    mipmap: MipMapDescriptor,

    // Boo (also sets Texture3D as !Sync and !Send)
    _phantom: PhantomData<*const T>,
}

impl<T: Texel> ToGlName for Texture3D<T> {
    fn name(&self) -> u32 {
        self.name
    }
}

impl<T: Texel> ToGlTarget for Texture3D<T> {
    fn target() -> u32 {
        gl::TEXTURE_3D
    }
}

impl<T: Texel> Drop for Texture3D<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.name);
        }
    }
}

impl<T: Texel> Texture for Texture3D<T> {
    type Region = (vek::Vec3<u16>, vek::Extent3<u16>);
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
        let extent = extent.as_::<i32>();
        gl::TextureStorage2D(name, levels as i32, T::INTERNAL_FORMAT, extent.w, extent.h);

        if ptr != null() {
            gl::TextureSubImage3D(
                name,
                0,
                0,
                0,
                0,
                extent.w,
                extent.h,
                extent.d,
                T::FORMAT,
                T::TYPE,
                ptr as *const c_void,
            );
        }
    }

    unsafe fn alloc_resizable_storage(
        name: u32,
        extent: <Self::Region as Region>::E,
        unique_level: u8,
        ptr: *const <Self::T as Texel>::Storage,
    ) {
        let extent = extent.as_::<i32>();
        gl::BindTexture(gl::TEXTURE_3D, name);
        gl::TexImage3D(
            gl::TEXTURE_2D,
            unique_level as i32,
            T::INTERNAL_FORMAT as i32,
            extent.w,
            extent.h,
            extent.d,
            0,
            T::FORMAT,
            T::TYPE,
            ptr as *const c_void,
        );
    }

    unsafe fn update_subregion(
        name: u32,
        region: Self::Region,
        level: u8,
        ptr: *const <Self::T as Texel>::Storage,
    ) {
        let origin = region.origin().as_::<i32>();
        let extent = region.extent().as_::<i32>();
        gl::TextureSubImage3D(
            name,
            level as i32,
            origin.x,
            origin.y,
            origin.z,
            extent.w,
            extent.h,
            extent.d,
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
            extent.d,
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
            extent.d,
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

    unsafe fn copy_subregion_from(
        name: u32,
        other_name: u32,
        level: u8,
        other_level: u8,
        region: Self::Region,
        offset: <Self::Region as Region>::O,
    ) {
        let origin = region.origin().as_::<i32>();
        let extent = region.extent().as_::<i32>();
        let offset = offset.as_::<i32>();

        gl::CopyImageSubData(
            other_name,
            gl::TEXTURE_2D,
            other_level as i32,
            origin.x,
            origin.y,
            origin.z,
            name,
            gl::TEXTURE_2D,
            level as i32,
            offset.x,
            offset.y,
            offset.z,
            extent.w,
            extent.h,
            extent.d,
        );
    }
}

impl<T: Texel> MultiLayerTexture for Texture3D<T> {
    fn is_layer_valid(&self, layer: u16) -> bool {
        layer < self.dimensions().d
    }
}
