use super::{convert_level_count, create_texture_raw, Sampler, TexelLayout, Texture, TextureMode};
use crate::{
    context::Cached,
    object::{Active, Bind, ToGlName, ToGlType},
};
use std::{
    marker::PhantomData,
    num::{NonZeroU32, NonZeroU8},
    ptr::{null, NonNull},
    rc::Rc,
};

// A 2D texture that will be used for rendering objects
pub struct Texture2D<T: TexelLayout> {
    // Internal OpenGL shit
    texture: NonZeroU32,

    // Main texture settings
    dimensions: vek::Extent2<u16>,
    mode: TextureMode,
    levels: NonZeroU8,
    sampler: Sampler,

    // Boo (also sets Texture2D as !Sync and !Send)
    _phantom: PhantomData<*const T>,
}

impl<T: TexelLayout> Cached for Texture2D<T> {}

impl<T: TexelLayout> ToGlName for Texture2D<T> {
    fn name(&self) -> NonZeroU32 {
        self.texture
    }
}

impl<T: TexelLayout> ToGlType for Texture2D<T> {
    fn target(&self) -> u32 {
        gl::TEXTURE_2D
    }
}

impl<T: TexelLayout> Bind for Texture2D<T> {
    fn bind(&mut self, _ctx: &mut crate::context::Context, function: impl FnOnce(crate::object::Active<Self>)) {
        unsafe {
            let target = self.target();
            gl::BindTexture(target, self.target());
            function(Active::new(self, _ctx));
        }
    }
}

impl<T: TexelLayout> Texture for Texture2D<T> {
    type Layout = T;

    type Dimensions = vek::Extent2<u16>;

    type Region = (vek::Vec2<u16>, vek::Extent2<u16>);

    fn dimensions(&self) -> Self::Dimensions {
        self.dimensions
    }

    fn region(&self) -> Self::Region {
        (vek::Vec2::zero(), self.dimensions)
    }

    fn mode(&self) -> super::TextureMode {
        self.mode
    }

    fn get_layer(&self, level: u8) -> Option<super::MipLayerRef<Self>> {
        (level < self.levels.get()).then(|| super::MipLayerRef::new(self, level))
    }

    fn get_layer_mut(&mut self, level: u8) -> Option<super::MipLayerMut<Self>> {
        (level < self.levels.get()).then(|| super::MipLayerMut::new(self, level))
    }

    unsafe fn clear_mip_layer_unchecked(&mut self, ctx: &mut crate::context::Context, level: u8, val: Self::Layout, region: Self::Region) {
        let format_ = Self::Layout::FORMAT;
        let type_ = gl::UNSIGNED_BYTE;
        let offset = region.0.as_();
        let extent = region.1.as_();
        gl::ClearTexSubImage(
            self.name().get(),
            level as i32,
            offset.x,
            offset.y,
            0,
            extent.w,
            extent.h,
            1,
            format_,
            type_,
            &val as *const Self::Layout as _,
        );
    }

    unsafe fn update_mip_layer_unchecked(&mut self, ctx: &mut crate::context::Context, level: u8, ptr: *const Self::Layout, region: Self::Region) {
        let format_ = Self::Layout::FORMAT;
        let type_ = gl::UNSIGNED_BYTE;
        let offset = region.0.as_();
        let extent = region.1.as_();
        gl::TextureSubImage2D(self.name().get(), level as i32, offset.x, offset.y, extent.w, extent.h, format_, type_, ptr as _)
    }

    unsafe fn read_mip_layer_unchecked(&self, ctx: &crate::context::Context, level: u8, out: *mut Self::Layout, region: Self::Region) {
        let format_ = Self::Layout::FORMAT;
        let type_ = gl::UNSIGNED_BYTE;
        let offset = region.0.as_();
        let extent = region.1.as_();
        let buf_size = i32::try_from(extent.as_::<u32>().product() * Self::Layout::bytes()).unwrap();
        gl::GetTextureSubImage(
            self.name().get(),
            level as i32,
            offset.x,
            offset.y,
            0,
            extent.w,
            extent.h,
            1,
            format_,
            type_,
            buf_size,
            out as _,
        )
    }

    unsafe fn from_raw_parts(
        ctx: &mut crate::context::Context,
        mode: TextureMode,
        sampling: super::Sampling,
        dimensions: Self::Dimensions,
        levels: NonZeroU8,
        ptr: Option<*const T>,
    ) -> Self {
        // Create a new raw OpenGL texture object
        let tex = create_texture_raw();

        // Check for mipmaps
        let mipmaps = convert_level_count(levels);

        // Convert dimensions
        let width = dimensions.w as i32;
        let height = dimensions.h as i32;

        // Pre-allocate storage using the texture mode (immutable vs mutable textures)
        match mode {
            TextureMode::Dynamic => {
                // Initialize the storage
                gl::TextureStorage2D(tex.get(), mipmaps.1 as i32, T::INTERNAL_FORMAT, dimensions.w as i32, dimensions.h as i32);

                // Fill the storage (only if the pointer is valid)
                if let Some(ptr) = ptr {
                    gl::TextureSubImage2D(tex.get(), 0, 0, 0, width, height, T::FORMAT, gl::UNSIGNED_BYTE, ptr as _);
                }
            }
            TextureMode::Resizable => {
                // Bind the texture (only for resizable textures tho)
                gl::BindTexture(gl::TEXTURE_2D, tex.get());

                // Initialize the texture with the valid data
                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    levels.get() as _,
                    T::INTERNAL_FORMAT as i32,
                    width,
                    height,
                    0,
                    T::FORMAT,
                    gl::UNSIGNED_BYTE,
                    ptr.unwrap_or_else(null) as _,
                )
            }
        }

        // Appply the sampling parameters and create a new sampler
        let sampler = super::apply(ctx, tex, gl::TEXTURE_2D, mode, sampling);

        // Create the texture wrapper
        Texture2D {
            texture: tex,
            dimensions,
            mode,
            levels,
            _phantom: Default::default(),
            sampler,
        }
    }

    fn sampler(&self) -> &super::Sampler {
        &self.sampler
    }
}
