use super::{Bindless, Sampler, TexelLayout};
use crate::{
    context::Context,
    object::{Bind, ToGlName, ToGlType},
};
use std::{
    marker::PhantomData,
    num::{NonZeroU32, NonZeroU8},
    ptr::NonNull,
};

// This will create a raw OpenGL texture
pub(super) unsafe fn create_texture_raw() -> NonZeroU32 {
    let mut tex = 0u32;
    gl::GenTextures(1, &mut tex);
    NonZeroU32::new(tex).unwrap()
}

// Convert some mip-map level counts into a bool and a u8
pub(super) unsafe fn convert_level_count(levels: NonZeroU8) -> (bool, u8) {
    let levels = levels.get();
    let mipmaps = levels > 1;
    (mipmaps, levels)
}

// Some settings that tell us exactly how we should generate a texture
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextureMode {
    // Dynamic textures can be modified throughout their lifetime, but they cannot change size
    Dynamic,

    // Resizable textures are just dynamic textures that can change their size at will
    Resizable,
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
        assert!((data.len() as u32) < self.texture.texel_count(), "Current length and output length do not match up.");

        // Le update texture subimage
        unsafe {
            self.update_unchecked(ctx, region, data);
        }
    }
}

// Texture dimensions trait. This is going to be implemented for vek::Extent2 and vek::Extent3
pub trait Dim: Copy {
    // Count the number of texels
    fn texel_count(&self) -> u32;

    // Check if the dimensions can be used to create a texture
    fn valid(&self) -> bool;

    // Get the max element from these dimensions
    fn max(&self) -> u16;

    // Caclulate the number of mipmap layers that a texture can have
    fn levels(&self) -> NonZeroU8 {
        let mut cur = self.max() as f32;
        let mut num = 0u32;
        while cur > 1.0 {
            cur /= 2.0;
            num += 1;
        }
        NonZeroU8::new(u8::try_from(num).unwrap()).unwrap()
    }
}

impl Dim for vek::Extent2<u16> {
    fn texel_count(&self) -> u32 {
        self.as_::<u32>().product()
    }

    fn valid(&self) -> bool {
        *self == vek::Extent2::zero()
    }

    fn max(&self) -> u16 {
        self.reduce_max()
    }
}

impl Dim for vek::Extent3<u16> {
    fn texel_count(&self) -> u32 {
        self.as_::<u32>().product()
    }

    fn valid(&self) -> bool {
        *self == vek::Extent3::zero()
    }

    fn max(&self) -> u16 {
        self.reduce_max()
    }
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
    fn new(ctx: &mut Context, mode: TextureMode, dimensions: Self::Dimensions, sampling: super::Sampling, mipmaps: bool, data: &[Self::Layout]) -> Option<Self> {
        // Validate the dimensions (make sure they aren't zero in ANY axii)
        let dims_valid = dimensions.valid();

        // Validate length (make sure the data slice matches up with dimensions)
        let len_valid = if !data.is_empty() {
            data.len() as u64 == (dimensions.texel_count() as u64) * (Self::Layout::bytes() as u64)
        } else {
            true
        };

        // Create the texture if the requirements are all valid
        (dims_valid && len_valid).then(|| unsafe {
            // Convert some parameters to their raw counterpart
            let ptr = (!data.is_empty()).then(|| data.as_ptr());
            let levels = mipmaps.then(|| dimensions.levels()).unwrap_or(NonZeroU8::new_unchecked(1));

            // Create the texture
            Self::from_raw_parts(ctx, mode, sampling, dimensions, levels, ptr)
        })
    }

    // Create the texture from it's raw parts, like levels and pointer
    unsafe fn from_raw_parts(
        ctx: &mut Context,
        mode: TextureMode,
        sampling: super::Sampling,
        dimensions: Self::Dimensions,
        levels: NonZeroU8,
        ptr: Option<*const Self::Layout>,
    ) -> Self;

    // Get the texture's dimensions
    fn dimensions(&self) -> Self::Dimensions;

    // Get the texture's region
    fn region(&self) -> Self::Region;

    // Get the texture's mode
    fn mode(&self) -> TextureMode;

    // Get this texture's unique sampler immutably
    fn sampler(&self) -> &Sampler;

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
