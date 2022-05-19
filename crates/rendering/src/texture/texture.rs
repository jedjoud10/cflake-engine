use crate::{
    context::Context,
    object::{Bind, ToGlName, ToGlType},
};
use std::{marker::PhantomData, num::{NonZeroU32, NonZeroU8}};
use super::TexelLayout;

// This will create a raw OpenGL texture
pub(super) unsafe fn create_texture_raw() -> NonZeroU32 {
    let mut tex = 0u32;
    gl::GenTextures(1, &mut tex);
    NonZeroU32::new(tex).unwrap()
}

// Some settings that tell us exactly how we should generate a texture
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextureMode {
    // Dynamic textures can be modified throughout their lifetime, but they cannot change size
    Dynamic,

    // Resizable textures are just dynamic textures that can change their size at will
    Resizable,

    // A bindless dynamic texture
    Bindless,
}

// An immutable mip layer that we can use to read from the texture
pub struct MipLayerRef<'a, T: Texture> {
    // El texture
    texture: &'a T,

    // The level of the mip layer
    level: u8,
}

impl<'a, T: Texture> MipLayerRef<'a, T> {
    // Create a new mip layer view using a texture and a level
    pub(super) fn new(texture: &'a T, level: u8) -> Self {
        Self { texture, level }
    }
}

// A mutable mip layer that we can use to write to the texture
pub struct MipLayerMut<'a, T: Texture> {
    // El texture
    texture: &'a mut T,

    // The level of the mip layer
    level: u8,
}


impl<'a, T: Texture> MipLayerMut<'a, T> {
    // Create a new mip layer mutable view using a texture and a level
    pub(super) fn new(texture: &'a mut T, level: u8) -> Self {
        Self { texture, level }
    }

    // Update a sub-region of the mip-layer, but without checking for safety
    unsafe fn update_unchecked(&mut self, ctx: &mut Context, region: T::Region, data: &[T::Layout]) {
        self.texture.update_mip_layer_unchecked(ctx, self.level, data.as_ptr(), region);
    }

    // Update a sub-region of the mip-layer using a data slice
    fn update(&mut self, ctx: &mut Context, region: T::Region, data: &[T::Layout]) {
        // Length should never be greater
        assert!((data.len() as u32) < self.texture.count_texels(), "Current length and output length do not match up.");

        // Le update texture subimage
        unsafe {
            self.update_unchecked(ctx, region, data);
        }
    } 
}

// Texture dimensions trait. This is going to be implemented for vek::Extent2 and vek::Extent3
pub trait Dim {
    // Count the number of texels
    fn texel_count(&self) -> u32;

    // Check if the dimensions can be used to create a texture
    fn valid(&self) -> bool;
}

// A global texture trait that will be implemented for Texture2D and ArrayTexture2D
pub trait Texture: ToGlName + ToGlType + Bind + Sized {
    // Output texel layout
    type Layout: TexelLayout;

    // Textures can have different dimensions
    type Dimensions: Dim;
    
    // A region that might fill the texture, like a rectangle for 2d textures and cubes for 3d textures
    type Region;

    // Create a new texutre that contains some data
    fn new(ctx: &mut Context, mode: TextureMode, dimensions: Self::Dimensions, levels: NonZeroU8, data: &[Self::Layout]) -> Option<Self> {
        // Validate texture parameters
        let valid = dimensions.valid();

        // Create texture
        valid.then(|| unsafe {
            Self::new_unchecked(ctx, mode, dimensions, levels, data)
        })
    }

    // Create a new texture that contains some data without checking for safety
    unsafe fn new_unchecked(ctx: &mut Context, mode: TextureMode, dimensions: Self::Dimensions, levels: NonZeroU8, data: &[Self::Layout]) -> Self;

    // Get the texture's dimensions
    fn dimensions(&self) -> Self::Dimensions;

    // Get the texture's region
    fn region(&self) -> Self::Region;

    // Get the texture's mode
    fn mode(&self) -> TextureMode;

    // Calculate the number of texels that make up this texture
    fn texel_count(&self) -> u32 {
        self.dimensions().texel_count()
    }

    // Get a single mip level from the texture, immutably
    fn get_layer(&self, level: u8) -> Option<MipLayerRef<Self>>;

    // Get a single mip level from the texture, mutably
    fn get_layer_mut(&mut self, level: u8) -> Option<MipLayerMut<Self>>;

    // Calculate the uncompressed size of the texture
    fn byte_count(&self) -> u64 {
        u64::from(Self::Layout::bytes()) * u64::from(self.texel_count())
    }

    // Clear a region of a mip layer by filling it with a constant value, without checking anything
    unsafe fn clear_mip_layer_unchecked(&mut self, ctx: &mut Context, level: u8, val: Self::Layout, region: Self::Region);

    // Update the contents of a whole single mip layer without checking anything
    unsafe fn update_mip_layer_unchecked(&mut self, ctx: &mut Context, level: u8, ptr: *const Self::Layout, region: Self::Region);

    // Read the contents of a whole single mip layer into an array
    unsafe fn read_mip_layer_unchecked(&self, ctx: &Context, level: u8, out: *mut Self::Layout, region: Self::Region);
}
