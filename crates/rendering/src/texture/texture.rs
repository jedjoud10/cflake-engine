use super::{
    get_bit, set_bit, Extent, Filter, MipLevelMut, MipLevelRef, MipMapDescriptor, MipMapSetting,
    Region, Sampling, Texel, TextureMode, Wrap,
};
use crate::{
    context::{Context, ToGlName, ToGlTarget},
    others::Comparison,
};
use std::{cell::Cell, mem::transmute, num::NonZeroU8, ptr::null, rc::Rc};
// A global texture trait that will be implemented for all our texture variants
// This texture trait does not allow us to fetch multiple layers in the case of multi-layered textures
// However, the Region's Origin for this texture must be a 3D vector if it is a multi-layered texture
// TODO: Test texture resizing with mipmapping, does it reallocate or not?
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
        mipmaps: MipMapSetting,
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
            let levels = match mipmaps {
                MipMapSetting::Disabled => NonZeroU8::new(1).unwrap(),
                MipMapSetting::Automatic => auto,
                MipMapSetting::Manual { levels } => levels.min(auto),
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
                    Self::alloc_resizable_storage(tex, dimensions, 0, ptr as _);

                    // Resizable texture are kinda goofy when dealing with manual mipmaps
                    if levels.get() > 1 {
                        gl::TextureParameteri(tex, gl::TEXTURE_MAX_LEVEL, levels.get() as i32);
                    }
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

                // Set the anisotropic samples
                if let Some(samples) = sampling.mipmap_aniso_samples {
                    gl::TextureParameterf(
                        tex,
                        gl::TEXTURE_MAX_ANISOTROPY_EXT,
                        samples.get() as f32,
                    );
                }

                // Set the LOD bias/range parameters
                gl::TextureParameterf(tex, gl::TEXTURE_LOD_BIAS, sampling.mipmap_lod_bias);
                gl::TextureParameterf(tex, gl::TEXTURE_MIN_LOD, sampling.mipmap_lod_range.0);
                gl::TextureParameterf(tex, gl::TEXTURE_MAX_LOD, sampling.mipmap_lod_range.1);
            }

            // Apply the comparison texture (only if we are using depth texels)
            let depth = <Self::T as Texel>::FORMAT == gl::DEPTH_COMPONENT;
            if let (Some(comparison), true) = (sampling.depth_comparison, depth) {
                gl::TextureParameteri(
                    tex,
                    gl::TEXTURE_COMPARE_MODE,
                    gl::COMPARE_REF_TO_TEXTURE as i32,
                );
                gl::TextureParameteri(
                    tex,
                    gl::TEXTURE_COMPARE_FUNC,
                    transmute::<Comparison, u32>(comparison) as i32,
                );
            }

            // Create a mip map accessor
            let mipmap = MipMapDescriptor {
                levels,
                read: Rc::new(Cell::new(0)),
                write: Rc::new(Cell::new(0)),
            };

            // Create the texture object
            Self::from_raw_parts(tex, dimensions, mode, mipmap)
        })
    }

    // Get the texture's region (origin state is default)
    fn region(&self) -> Self::Region {
        Self::Region::with_extent(self.dimensions())
    }

    // Checks if we can modify a region of the texture
    fn is_region_valid(&self, region: Self::Region) -> bool {
        let extent =
            <Self::Region as Region>::extent_from_origin(region.origin()) + region.extent();
        extent.is_self_smaller(self.dimensions())
    }

    // Get the texture's dimensions
    fn dimensions(&self) -> <Self::Region as Region>::E;

    // Get the dimensions of a specific mip level in this texture
    fn dimensions_of_level(&self, level: u8) -> Option<<Self::Region as Region>::E> {
        if level == 0 {
            Some(self.dimensions())
        } else if level < self.levels() {
            Some(unsafe {
                <<Self::Region as Region>::E as Extent>::get_level_extent(self.name(), level)
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

    // Get the inner mipmap accessor for this texture
    fn mipmap_descriptor(&self) -> &MipMapDescriptor;

    // Check if the user is currently writing to the texture in any way
    fn is_user_writing(&self) -> bool {
        self.mipmap_descriptor().write.get() != 0
    }

    // Get a single mip level from the texture, immutably
    // This will fail if the mip level is currently being used mutably
    fn mip(&self, level: u8) -> Option<MipLevelRef<Self>> {
        if level > self.levels() {
            return None;
        }

        if get_bit(&self.mipmap_descriptor().write, level) {
            return None;
        }

        set_bit(&self.mipmap_descriptor().read, level, true);

        Some(MipLevelRef::new(
            self,
            level,
            self.mipmap_descriptor().read.clone(),
        ))
    }

    // Get a single mip level from the texture, mutably (uses internal mutability pattern)
    // This will fail if the mip level is currently being used mutably or being read from
    fn mip_mut(&self, level: u8) -> Option<MipLevelMut<Self>> {
        if level > self.levels() {
            return None;
        }

        if get_bit(&self.mipmap_descriptor().write, level)
            || get_bit(&self.mipmap_descriptor().read, level)
        {
            return None;
        }

        set_bit(&self.mipmap_descriptor().read, level, true);
        set_bit(&self.mipmap_descriptor().write, level, true);

        Some(MipLevelMut::new(
            self,
            level,
            self.mipmap_descriptor().read.clone(),
            self.mipmap_descriptor().write.clone(),
        ))
    }

    // Get the number of mipmap levels we are using
    fn levels(&self) -> u8 {
        self.mipmap_descriptor().levels.get()
    }

    // Automatically regenerate the texture's miplevels using an OpenGL function
    fn generate_mipmaps(&mut self) -> bool {
        if self.levels() > 0 {
            unsafe {
                gl::GenerateTextureMipmap(self.name());
            }
            true
        } else {
            false
        }
    }

    // Resize the current texture (this will also set it's inner data to null)
    // This will panic if we try to resize a static texture
    fn resize(&mut self, extent: <Self::Region as Region>::E) {
        assert!(
            self.mode().resize_permission(),
            "Cannot resize texture, missing permission"
        );
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
        mipmap: MipMapDescriptor,
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

    // Copy a sub-region of another texture into this texture
    unsafe fn copy_subregion_from(
        write_name: u32,
        read_name: u32,
        write_level: u8,
        read_level: u8,
        read_region: Self::Region,
        write_offset: <Self::Region as Region>::O,
    );
}

// Implemented for textures that have only one single layer
pub trait SingleLayerTexture: Texture {}

// Implemented for textures that have multiple layers
pub trait MultiLayerTexture: Texture {
    // Check if the texture contains the specific layer
    fn is_layer_valid(&self, layer: u16) -> bool;
}
