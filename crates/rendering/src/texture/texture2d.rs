use assets::Asset;

use super::{
    ImageTexel, MipMapDescriptor, Region, Texel, Texture, TextureImportSettings, TextureMode, DepthTexel,
};
use crate::{context::{Context, ToGlName, ToGlTarget}, others::Comparison};
use std::{ffi::c_void, marker::PhantomData, ptr::null};

// A 2D texture that contains multiple pixels that have their own channels
// Each pixel can be either a single value, RG, RGB, or even RGBA
// These individual pixels are called texels, since they are used within the texture
pub struct Texture2D<T: Texel> {
    // Internal OpenGL shit
    name: u32,

    // Main texture settings
    dimensions: vek::Extent2<u16>,
    mode: TextureMode,
    mipmap: MipMapDescriptor,

    // Boo (also sets Texture2D as !Sync and !Send)
    _phantom: PhantomData<*const T>,
}

impl<T: Texel> ToGlName for Texture2D<T> {
    fn name(&self) -> u32 {
        self.name
    }
}

impl<T: Texel> ToGlTarget for Texture2D<T> {
    fn target() -> u32 {
        gl::TEXTURE_2D
    }
}

impl<T: Texel> Drop for Texture2D<T> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.name);
        }
    }
}

impl<T: Texel> Texture for Texture2D<T> {
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
        let extent = extent.as_::<i32>();
        gl::TextureStorage2D(
            name,
            levels as i32,
            T::INTERNAL_FORMAT,
            extent.w,
            extent.h,
        );

        if ptr != null() {
            gl::TextureSubImage2D(
                name,
                0,
                0,
                0,
                extent.w,
                extent.h,
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
        gl::BindTexture(gl::TEXTURE_2D, name);
        gl::TexImage2D(
            gl::TEXTURE_2D,
            unique_level as i32,
            T::INTERNAL_FORMAT as i32,
            extent.w,
            extent.h,
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
        gl::TextureSubImage2D(
            name,
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
            0,
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
            0,
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
            gl::TEXTURE_2D,
            other_level as i32,
            origin.x,
            origin.y,
            0,
            name,
            gl::TEXTURE_2D,
            level as i32,
            offset.x,
            offset.y,
            0,
            extent.w,
            extent.h,
            1,
        );
    }
}

impl<'a, T: ImageTexel> Asset<'a> for Texture2D<T> {
    type Args = (&'a mut Context, TextureImportSettings);

    fn extensions() -> &'static [&'static str] {
        &["png", "jpg"]
    }

    fn deserialize(data: assets::Data, args: Self::Args) -> Self {
        let (ctx, settings) = args;
        let image = image::load_from_memory(data.bytes()).unwrap();
        let dimensions = vek::Extent2::new(image.width() as u16, image.height() as u16);
        /*
        let image = match settings.scale {
            super::TextureScale::Default => image,
            super::TextureScale::Scale { scaling, filter } => image.resize((dimensions.w as f64 * scaling) as u32, (dimensions.h as f64 * scaling) as u32, filter),
            super::TextureScale::Resize { size, filter } => image.resize(size.w.get() as u32, size.h.get() as u32, filter),
        };
        
        let image = if !settings.flip.y {
            image.flipv()
        } else {            
            image
        };

        let image = if settings.flip.x {
            image.fliph()
        } else {
            image
        };
        */

        let dimensions = vek::Extent2::new(image.width() as u16, image.height() as u16);
        let texels = T::to_image_texels(image);
        Self::new(
            ctx,
            settings.mode,
            dimensions,
            settings.sampling,
            settings.mipmaps,
            Some(texels.as_slice()),
        )
        .unwrap()
    }
}