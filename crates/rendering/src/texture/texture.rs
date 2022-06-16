use super::{Bindless, Sampler, Texel, Sampling, Wrap, Filter};
use crate::{
    context::Context,
    object::{ToGlName, ToGlTarget},
};
use std::{num::NonZeroU8, ptr::null, rc::Rc, ffi::c_void};

// Some settings that tell us exactly how we should generate a texture
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TextureMode {
    // Static textures cannot be modified, they can only be read
    Static,

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
    unsafe fn update_unchecked(
        &mut self,
        _ctx: &mut Context,
        region: T::Region,
        data: &[<T::T as Texel>::Storage],
    ) {
        T::update_subregion(self.texture.name(), region, data.as_ptr() as _)
    }

    // Update a sub-region of the mip-layer using a data slice
    fn update(
        &mut self,
        ctx: &mut Context,
        region: T::Region,
        data: &[<T::T as Texel>::Storage],
    ) {
        // The length of the buffer should be equal to the surface area of the region
        assert!(
            (data.len() as u32) == region.area(),
            "Input data length is not equal to region area surface"
        );

        // Le update texture subimage
        unsafe {
            self.update_unchecked(ctx, region, data);
        }
    }
}

// Texture dimensions traits that are simply implemented for extents
pub trait Extent: Copy {
    // Get the surface area of a superficial rectangle that uses these extents as it's dimensions
    fn area(&self) -> u32;

    // Check if this region can be used to create a texture or update it
    fn is_valid(&self) -> bool;

    // Get the max element from these dimensions
    fn max(&self) -> u16;

    // Caclulate the number of mipmap layers that a texture can have (assuming that the offset is 0)
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

// Implementations of extent for 2D and 3D extents
impl Extent for vek::Extent2<u16> {
    fn area(&self) -> u32 {
        self.as_::<u32>().product()
    }

    fn is_valid(&self) -> bool {
        *self != vek::Extent2::zero()
    }

    fn max(&self) -> u16 {
        self.reduce_max()
    }
}

impl Extent for vek::Extent3<u16> {
    fn area(&self) -> u32 {
        self.as_::<u32>().product()
    }

    fn is_valid(&self) -> bool {
        *self != vek::Extent3::zero()
    }

    fn max(&self) -> u16 {
        self.reduce_max()
    }
}

// Texture region trait that will be implemented for (origin, extent) tuples
pub trait Region {
    // Regions are defined by their origin and extents
    type O: Default + Copy;
    type E: Copy + Extent;

    // Create a texel region of one singular unit (so we can store a singular texel)
    fn unit() -> Self;

    // Get the region's origin
    fn origin(&self) -> &Self::O;

    // Get the region's extent
    fn extent(&self) -> &Self::E;

    // Create a region with a default origin using an extent
    fn with_extent(extent: Self::E) -> Self;

    // Calculate the surface area of the region
    fn area(&self) -> u32;
}

impl Region for (vek::Vec2<u16>, vek::Extent2<u16>) {
    type O = vek::Vec2<u16>;
    type E = vek::Extent2<u16>;

    fn origin(&self) -> &Self::O {
        &self.0
    }

    fn extent(&self) -> &Self::E {
        &self.1
    }

    fn with_extent(extent: Self::E) -> Self {
        (Default::default(), extent)
    }

    fn area(&self) -> u32 {
        self.extent().area()
    }

    fn unit() -> Self {
        (vek::Vec2::zero(), vek::Extent2::one())
    }
}

impl Region for (vek::Vec3<u16>, vek::Extent3<u16>) {
    type O = vek::Vec3<u16>;
    type E = vek::Extent3<u16>;

    fn origin(&self) -> &Self::O {
        &self.0
    }

    fn extent(&self) -> &Self::E {
        &self.1
    }

    fn with_extent(extent: Self::E) -> Self {
        (Default::default(), extent)
    }

    fn area(&self) -> u32 {
        self.extent().area()
    }

    fn unit() -> Self {
        (vek::Vec3::zero(), vek::Extent3::one())
    }
}

// This enum tells the texture how exactly it should create it's mipmaps
// Default mode for mipmap generation is MipMaps::AutomaticAniso
pub enum MipMaps {
    // Disable mipmap generation for the texture
    Disabled,

    // Automatic mipmap generation based on the texture dimensions
    Automatic,

    // Manual mipmap generation with specific levels.
    // This will be clamped to the maximum number of levels allowed for the given texture dimensions
    // If levels is less than 2, then mipmapping will be disabled
    Manual {
        levels: NonZeroU8
    },

    // Automatic mipmap generation (from texture dimensions), but with a specified number of anisotropy samples
    // If samples is less than 2m then anisotropic filtering will be disabled
    AutomaticAniso {
        samples: NonZeroU8,
    },

    // Manual mipmap generation, but with a specified number of anisotropy sampler
    // If levels is less than 2, then mipmapping will be disabled
    // If samples is less than 2m then anisotropic filtering will be disabled
    ManualAniso {
        levels: NonZeroU8,
        samples: NonZeroU8,
    }
}

impl Default for MipMaps {
    fn default() -> Self {
        Self::AutomaticAniso { samples: NonZeroU8::new(16).unwrap() }
    }
}

// A global texture trait that will be implemented for Texture2D and ArrayTexture2D
pub trait Texture: ToGlName + ToGlTarget + Sized {
    // Texel region (position + extent)
    type Region: Region;

    // Texel layout that we will use internally
    type T: Texel;
    // Create a new texture that contains some data
    fn new(
        ctx: &mut Context,
        mode: TextureMode,
        dimensions: <Self::Region as Region>::E,
        sampling: Sampling,
        mipmaps: MipMaps, 
        data: &[<Self::T as Texel>::Storage],
    ) -> Option<Self> {
        // Validate the dimensions (make sure they aren't zero in ANY axii)
        let dims_valid = dimensions.is_valid();

        // Validate length (make sure the data slice matches up with dimensions)
        let len_valid = if !data.is_empty() {
            data.len() as u64 == (dimensions.area() as u64)
        } else {
            true
        };

        // Create the texture if the requirements are all valid
        (dims_valid && len_valid).then(|| unsafe {
            // Convert some parameters to their raw counterpart
            let ptr = (!data.is_empty())
                .then(|| data.as_ptr())
                .unwrap_or_else(null);

            // Calculate the total mipmap levels (and optionally the number of anisotropy samples)
            let auto = dimensions.levels();
            let (levels, anisotropy_samples) = match mipmaps {
                MipMaps::Disabled => (NonZeroU8::new(1).unwrap(), None),
                MipMaps::Automatic => (auto, None),
                MipMaps::Manual { levels } => (levels.min(auto), None),
                MipMaps::AutomaticAniso { samples } => (auto, Some(samples)),
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

            // Create a bindless handle for dynamic textures only (since dealing with resizable textures would be an absolute pain)
            let bindless = if mode == TextureMode::Dynamic {
                //super::create_bindless(ctx, tex, 200, mode)
                None
            } else {
                None
            };

            // Appply the sampling parameters for this texture
            // We do a bit of enum fetching (this is safe) (trust)
            let filter = std::mem::transmute::<Filter, u32>(sampling.filter);

            // Min and mag filters conversion cause OpenGL suxs
            let min = filter as i32;
            let mag = filter as i32;
                
            // Set the filters
            gl::TextureParameteri(tex, gl::TEXTURE_MIN_FILTER, min);
            gl::TextureParameteri(tex, gl::TEXTURE_MAG_FILTER, mag);
                
            // Convert the wrapping mode enum to the raw opengl type
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
        
            // Set the border color (if needed)
            if let Some(border) = border {
                gl::TextureParameterfv(tex, gl::TEXTURE_BORDER_COLOR, border.as_ptr());
            }

            // Apply the mipmapping settings (and anisostropic filtering)
            // This will automatically generate the 
            if levels.get() > 1 {
                gl::GenerateTextureMipmap(tex);
                if let Some(samples) = anisotropy_samples {
                    gl::TextureParameterf(tex, gl::TEXTURE_MAX_ANISOTROPY_EXT, samples.get() as f32);
                }
            }

            // Create the texture object
            Self::from_raw_parts(tex, dimensions, mode, levels, bindless)
        })
    }

    // Get the texture's region (origin state is default)
    fn region(&self) -> Self::Region {
        Self::Region::with_extent(self.dimensions())
    }

    // Get the texture's dimensions
    fn dimensions(&self) -> <Self::Region as Region>::E;

    // Get the texture's mode
    fn mode(&self) -> TextureMode;

    // Create an immutable texture sampler
    fn sampler(&self) -> Sampler<Self>;

    // Get the bindless handle that is stored within this texture
    fn bindless(&self) -> Option<&Bindless>;

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

    // Calculate the uncompressed size of the texture
    fn byte_count(&self) -> u64 {
        u64::from(Self::T::bytes()) * u64::from(self.texel_count())
    }

    // Force this texture to be stored within system memory (if it is a bindless texture)
    fn try_make_non_resident(&mut self) {
        if let Some(bindless) = self.bindless() {
            bindless.set_residency(false);
        }
    }

    // Force this texture to be stored within vram (if it is a bindless texture)
    fn try_make_resident(&mut self) {
        if let Some(bindless) = self.bindless() {
            bindless.set_residency(true);
        }
    }

    // Construct the texture object from it's raw parts
    unsafe fn from_raw_parts(
        name: u32,
        dimensions: <Self::Region as Region>::E,
        mode: TextureMode,
        levels: NonZeroU8,
        bindless: Option<Rc<Bindless>>,
    ) -> Self;

    // Allocate some immutable texture storage during texture initialization
    unsafe fn alloc_immutable_storage(
        name: u32,
        extent: <Self::Region as Region>::E,
        levels: u8,
        ptr: *const c_void,
    );

    // Allocate some mutable(resizable) texture during texture initialization
    // PS: This will allocate the texture storage for only one level
    unsafe fn alloc_resizable_storage(
        name: u32,
        extent: <Self::Region as Region>::E,
        unique_level: u8,
        ptr: *const c_void,
    );

    // Update a sub-region of the raw texture
    unsafe fn update_subregion(name: u32, region: Self::Region, ptr: *const c_void);
}
