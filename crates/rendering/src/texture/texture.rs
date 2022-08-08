use super::{Filter, MipMaps, Sampling, Texel, TextureMode, Wrap};
use crate::{
    context::Context,
    object::{ToGlName, ToGlTarget},
};
use std::{ffi::c_void, num::NonZeroU8, ptr::null};

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

    // Download a sub-region of the mip-layer, without checking for safety
    pub unsafe fn download_unchecked(
        &self,
        region: T::Region,
        data: *mut <T::T as Texel>::Storage,
    ) {
        T::read_subregion(self.texture.name(), region, self.level, data, region.area());
    }

    // Read the pixels from a layer (synchronous)
    pub fn download(&self, region: T::Region) -> Vec<<T::T as Texel>::Storage> {
        assert_ne!(region.area(), 0, "Input data length cannot be zero");

        let mut vec = Vec::<<T::T as Texel>::Storage>::with_capacity(region.area() as usize);
        unsafe {
            self.download_unchecked(region, vec.as_mut_ptr());
        }
        vec
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

    // Download a sub-region of the mip-layer, without checking for safety
    pub unsafe fn download_unchecked(
        &self,
        region: T::Region,
        data: *mut <T::T as Texel>::Storage,
    ) {
        T::read_subregion(self.texture.name(), region, self.level, data, region.area());
    }

    // Read the pixels from a layer (synchronous)
    pub fn download(&self, region: T::Region) -> Vec<<T::T as Texel>::Storage> {
        assert_ne!(region.area(), 0, "Input data length cannot be zero");

        let mut vec = Vec::<<T::T as Texel>::Storage>::with_capacity(region.area() as usize);
        unsafe {
            self.download_unchecked(region, vec.as_mut_ptr());
        }
        vec
    }

    // Update a sub-region of the mip-layer, but without checking for safety
    pub unsafe fn upload_unhecked(
        &mut self,
        region: T::Region,
        data: *const <T::T as Texel>::Storage,
    ) {
        T::update_subregion(self.texture.name(), region, data)
    }

    // Update a sub-region of the mip-layer using a data slice (synchronous)
    pub fn upload(&mut self, region: T::Region, data: &[<T::T as Texel>::Storage]) {
        assert!(
            (data.len() as u32) == region.area(),
            "Input data length is not equal to region area surface"
        );

        assert_ne!(data.len(), 0, "Input data length cannot be zero");

        // Le update texture subimage
        unsafe {
            self.upload_unhecked(region, data.as_ptr());
        }
    }

    // Read the pixels from a layer (synchronous)
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
        NonZeroU8::new(u8::try_from(num + 1).unwrap()).unwrap()
    }
}

// Implementation of extent for 2D extent
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

// Implementation of extent for 3D extent
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
pub trait Region: Copy {
    // Regions are defined by their origin and extents
    type O: Default + Copy;
    type E: Copy + Extent;

    // Create a texel region of one singular unit (so we can store a singular texel)
    fn unit() -> Self;

    // Get the region's origin
    fn origin(&self) -> Self::O;

    // Get the region's extent
    fn extent(&self) -> Self::E;

    // Create a region with a default origin using an extent
    fn with_extent(extent: Self::E) -> Self;

    // Calculate the surface area of the region
    fn area(&self) -> u32;
}

impl Region for (vek::Vec2<u16>, vek::Extent2<u16>) {
    type O = vek::Vec2<u16>;
    type E = vek::Extent2<u16>;

    fn origin(&self) -> Self::O {
        self.0
    }

    fn extent(&self) -> Self::E {
        self.1
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

    fn origin(&self) -> Self::O {
        self.0
    }

    fn extent(&self) -> Self::E {
        self.1
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
            gl::TextureParameteri(tex, gl::TEXTURE_MIN_FILTER, match (sampling.filter, levels.get() > 1) {
                (Filter::Nearest, false) => gl::NEAREST,
                (Filter::Linear, false) => gl::LINEAR,
                (Filter::Nearest, true) => gl::NEAREST_MIPMAP_NEAREST,
                (Filter::Linear, true) => gl::LINEAR_MIPMAP_LINEAR,
            } as i32);

            // Set the texture magnification filter
            gl::TextureParameteri(tex, gl::TEXTURE_MAG_FILTER, match sampling.filter {
                Filter::Nearest => gl::NEAREST,
                Filter::Linear => gl::LINEAR,
            } as i32);

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

    // Resize the current texture
    fn resize(&mut self, _extent: <Self::Region as Region>::E) {
        todo!()
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

    // Update a sub-region of the raw texture
    unsafe fn update_subregion(name: u32, region: Self::Region, ptr: *const <Self::T as Texel>::Storage);

    // Read a sub-region of the raw texture
    // PS: This will read the texture storage for only one level
    unsafe fn read_subregion(name: u32, region: Self::Region, level: u8, ptr: *mut <Self::T as Texel>::Storage, texels: u32);
}
