use super::{
    Extent, Filter, MipLayerMut, MipLayerRef, MipMaps, Region, Sampling, Texel, TextureMode, Wrap,
};
use crate::{
    context::Context,
    object::{ToGlName, ToGlTarget},
};
use std::{num::NonZeroU8, ptr::null};
// A global texture trait that will be implemented for Texture2D and ArrayTexture2D
// TODO: Test texture resizing with mipmapping, does it reallocate or not?
// TODO: Test texture mip map layer pixel reading / writing
pub trait Texture: ToGlName + ToGlTarget + Sized {
    // Texel region (position + extent)
    type Region: Region;

    // Texel layout that we will use internally
    type T: Texel;

    // Create a new texture that contains some predefined data
    fn new(
        _ctx: &mut Context,
        mode: TextureMode,
        dimensions: <Self::Region as Region>::E,
        sampling: Sampling,
        mipmaps: MipMaps,
        data: Option<&[<Self::T as Texel>::Storage]>,
    ) -> Option<Self> {
        // Validate the dimensions (make sure they aren't zero in ANY axii)
        let dims_valid = dimensions.is_valid();

        // Validate length (make sure the data slice matches up with dimensions)
        let len_valid = if let Some(data) = data {
            data.len() as u64 == (dimensions.area() as u64)
        } else {
            true
        };

        // Create the texture if the requirements are all valid
        (dims_valid && len_valid).then(|| unsafe {
            // Convert some parameters to their raw counterpart
            let ptr = data.map_or_else(null, |p| p.as_ptr());

            // Calculate the total mipmap levels (and optionally the number of anisotropy samples)
            let auto = dimensions.levels();
            let (levels, anisotropy_samples) = match mipmaps {
                MipMaps::Disabled => (NonZeroU8::new(1).unwrap(), None),
                MipMaps::Automatic => (auto, None),
                MipMaps::Manual { levels } => (levels.min(auto), None),
                MipMaps::AutomaticAniso => (auto, {
                    let mut val = 0.0;
                    gl::GetFloatv(gl::MAX_TEXTURE_MAX_ANISOTROPY_EXT, &mut val);
                    NonZeroU8::new(val as u8)
                }),
                MipMaps::ManualAniso { levels, samples } => (levels.min(auto), Some(samples)),
            };

            // Create a new raw OpenGL texture object
            let tex = {
                let mut tex = 0u32;
                gl::CreateTextures(Self::target(), 1, &mut tex);
                tex
            };

            // Pre-allocate storage using the texture mode (immutable vs mutable textures)
            match mode {
                TextureMode::Dynamic | TextureMode::Static => {
                    Self::alloc_immutable_storage(tex, dimensions, levels.get(), ptr as _)
                }
                TextureMode::Resizable => {
                    Self::alloc_resizable_storage(tex, dimensions, 0, ptr as _)
                }
            }

            // The texture minifcation filter
            gl::TextureParameteri(
                tex,
                gl::TEXTURE_MIN_FILTER,
                match (sampling.filter, levels.get() > 1) {
                    (Filter::Nearest, false) => gl::NEAREST,
                    (Filter::Linear, false) => gl::LINEAR,
                    (Filter::Nearest, true) => gl::NEAREST_MIPMAP_NEAREST,
                    (Filter::Linear, true) => gl::LINEAR_MIPMAP_LINEAR,
                } as i32,
            );

            // Set the texture magnification filter
            gl::TextureParameteri(
                tex,
                gl::TEXTURE_MAG_FILTER,
                match sampling.filter {
                    Filter::Nearest => gl::NEAREST,
                    Filter::Linear => gl::LINEAR,
                } as i32,
            );

            // Convert the wrapping mode enum to the raw OpenGL type
            let (wrap, border) = match sampling.wrap {
                Wrap::ClampToEdge => (gl::CLAMP_TO_EDGE, None),
                Wrap::ClampToBorder(b) => (gl::CLAMP_TO_BORDER, Some(b)),
                Wrap::Repeat => (gl::REPEAT, None),
                Wrap::MirroredRepeat => (gl::MIRRORED_REPEAT, None),
            };

            // Set the wrapping mode (for all 3 axii)
            gl::TextureParameteri(tex, gl::TEXTURE_WRAP_S, wrap as i32);
            gl::TextureParameteri(tex, gl::TEXTURE_WRAP_T, wrap as i32);
            gl::TextureParameteri(tex, gl::TEXTURE_WRAP_R, wrap as i32);

            // Set the border color if needed
            if let Some(border) = border {
                gl::TextureParameterfv(tex, gl::TEXTURE_BORDER_COLOR, border.as_ptr());
            }

            // Apply the mipmapping settings (and anisostropic filtering)
            if levels.get() > 1 {
                gl::GenerateTextureMipmap(tex);
                if let Some(samples) = anisotropy_samples {
                    gl::TextureParameterf(
                        tex,
                        gl::TEXTURE_MAX_ANISOTROPY_EXT,
                        samples.get() as f32,
                    );
                }
            }

            // Create the texture object
            Self::from_raw_parts(tex, dimensions, mode, levels)
        })
    }

    // Get the texture's region (origin state is default)
    fn region(&self) -> Self::Region {
        Self::Region::with_extent(self.dimensions())
    }

    // Get the texture's dimensions
    fn dimensions(&self) -> <Self::Region as Region>::E;

    // Get the dimensions of a specific mip layer in this texture
    fn dimensions_of_layer(&self, level: u8) -> Option<<Self::Region as Region>::E> {
        if level == 0 {
            Some(self.dimensions())
        } else if level < self.levels().get() {
            Some(unsafe { 
                <<Self::Region as Region>::E as Extent>::get_layer_extent(self.name(), level)
            })
        } else {
            None
        }
    }

    // Get the texture's mode
    fn mode(&self) -> TextureMode;

    // Calculate the number of texels that make up this texture
    fn texel_count(&self) -> u32 {
        self.dimensions().area()
    }

    // Get the number of mipmap layers that this texture uses
    fn levels(&self) -> NonZeroU8;

    // Get a single mip level from the texture, immutably
    fn get_layer(&self, level: u8) -> Option<MipLayerRef<Self>>;

    // Get a single mip level from the texture, mutably
    fn get_layer_mut(&mut self, level: u8) -> Option<MipLayerMut<Self>>;

    // Resize the current texture (this will also set it's inner data to null)
    fn resize(&mut self, extent: <Self::Region as Region>::E) {
        unsafe {
            Self::alloc_resizable_storage(self.name(), extent, 0, null());
        }
    }

    // Calculate the uncompressed size of the texture
    fn byte_count(&self) -> u64 {
        u64::from(Self::T::bytes()) * u64::from(self.texel_count())
    }

    // Construct the texture object from it's raw parts
    unsafe fn from_raw_parts(
        name: u32,
        dimensions: <Self::Region as Region>::E,
        mode: TextureMode,
        levels: NonZeroU8,
    ) -> Self;

    // Allocate some immutable texture storage during texture initialization
    unsafe fn alloc_immutable_storage(
        name: u32,
        extent: <Self::Region as Region>::E,
        levels: u8,
        ptr: *const <Self::T as Texel>::Storage,
    );

    // Allocate some mutable(resizable) texture during texture initialization
    // PS: This will allocate the texture storage for only one level
    unsafe fn alloc_resizable_storage(
        name: u32,
        extent: <Self::Region as Region>::E,
        level: u8,
        ptr: *const <Self::T as Texel>::Storage,
    );

    // Update a sub-region of a raw texture layer
    unsafe fn update_subregion(
        name: u32,
        region: Self::Region,
        level: u8,
        ptr: *const <Self::T as Texel>::Storage,
    );

    // Fills a sub-region of a raw texture layer with a constant value
    unsafe fn splat_subregion(
        name: u32,
        region: Self::Region,
        level: u8,
        ptr: *const <Self::T as Texel>::Storage,
    );

    // Fills the whole raw texture layer with a constant value
    unsafe fn splat(name: u32, level: u8, ptr: *const <Self::T as Texel>::Storage);

    // Read a sub-region of a raw texture layer
    unsafe fn read_subregion(
        name: u32,
        region: Self::Region,
        level: u8,
        ptr: *mut <Self::T as Texel>::Storage,
        texels: u32,
    );

    // Read the whole raw textrue layer
    unsafe fn read(name: u32, level: u8, ptr: *mut <Self::T as Texel>::Storage, texels: u32);
}
