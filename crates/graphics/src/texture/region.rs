use std::{num::NonZeroU8, ops::Add};
use vulkan::vk;

// Texture dimensions traits that are simply implemented for extents
pub trait Extent: Copy {
    // Get the surface area of a superficial rectangle that uses these extents as it's dimensions
    fn area(&self) -> u32;

    // Get the max element from these dimensions
    fn reduce_max(&self) -> u32;

    // Get the min element from these dimensions
    fn reduce_min(&self) -> u32;

    // Check if this region can be used to create a texture or update it
    fn is_valid(&self) -> bool {
        self.reduce_min() > 0        
    }

    // Caclulate the number of mipmap levels that a texture can have
    fn levels(&self) -> NonZeroU8 {
        let cur = self.reduce_max() as f32;
        let num = cur.log2().floor() + 1.0;
        NonZeroU8::new(u8::try_from(num as u8).unwrap())
            .unwrap_or(NonZeroU8::new(1).unwrap())
    }

    // Check if an extent is larger in all axii than another one
    fn is_larger_than(self, other: Self) -> bool;

    // Convert to a Vulkan Extent3D
    fn as_vk_extent(&self) -> vk::Extent3D;

    // Get the dimensionality of the extent (1, 2, or 3)
    fn dimensionality() -> usize;
}

// Implementation of extent for 2D extent
impl Extent for vek::Extent2<u32> {
    fn area(&self) -> u32 {
        self.product()
    }

    fn reduce_max(&self) -> u32 {
        vek::Extent2::reduce_max(*self)
    }

    fn reduce_min(&self) -> u32 {
        vek::Extent2::reduce_min(*self)
    }

    fn is_larger_than(self, other: Self) -> bool {
        self.cmpge(&other).reduce_and()
    }

    fn as_vk_extent(&self) -> vk::Extent3D {
        vk::Extent3D {
            width: self.w,
            height: self.h,
            depth: 1,
        }
    }

    fn dimensionality() -> usize {
        vek::Extent2::<u32>::ELEM_COUNT
    }
}

// Implementation of extent for 3D extent
impl Extent for vek::Extent3<u32> {
    fn area(&self) -> u32 {
        self.as_::<u32>().product()
    }

    fn is_valid(&self) -> bool {
        *self != vek::Extent3::zero()
    }

    fn reduce_max(&self) -> u32 {
        vek::Extent3::reduce_max(*self)
    }

    fn reduce_min(&self) -> u32 {
        vek::Extent3::reduce_max(*self)
    }

    fn is_larger_than(self, other: Self) -> bool {
        self.cmpge(&other).reduce_and()
    }

    fn as_vk_extent(&self) -> vk::Extent3D {
        vk::Extent3D {
            width: self.w,
            height: self.h,
            depth: self.h,
        }
    }

    fn dimensionality() -> usize {
        vek::Extent3::<u32>::ELEM_COUNT
    }
}

// Texture region trait that will be implemented for (origin, extent) tuples
pub trait Region: Copy {
    // Regions are defined by their origin and extents
    type O: Default + Copy + Add<Self::O, Output = Self::O>;
    type E: Extent + Copy + Add<Self::E, Output = Self::E> + PartialEq;

    // Create a texel region of one singular unit (so we can store a singular texel)
    fn unit() -> Self;

    // Get the region's origin
    fn origin(&self) -> Self::O;

    // Get the region's extent
    fn extent(&self) -> Self::E;

    // Set the region's origin
    fn set_origin(&mut self, origin: Self::O);

    // Set the region's extent
    fn set_extent(&mut self, extent: Self::E);

    // Create an extent from an origin
    fn extent_from_origin(origin: Self::O) -> Self::E;

    // Create a region with a default origin using an extent
    fn with_extent(extent: Self::E) -> Self;

    // Create a region with it's raw components
    fn from_raw_parts(origin: Self::O, extent: Self::E) -> Self;

    // Is this region a multi-layer region
    fn is_multi_layered() -> bool;

    // Calculate the surface area of the region
    fn area(&self) -> u32;
}

// Texture2D
impl Region for (vek::Vec2<u32>, vek::Extent2<u32>) {
    type O = vek::Vec2<u32>;
    type E = vek::Extent2<u32>;

    fn unit() -> Self {
        (vek::Vec2::zero(), vek::Extent2::one())
    }

    fn origin(&self) -> Self::O {
        self.0
    }

    fn extent(&self) -> Self::E {
        self.1
    }

    fn set_origin(&mut self, origin: Self::O) {
        self.0 = origin;
    }

    fn set_extent(&mut self, extent: Self::E) {
        self.1 = extent;
    }

    fn extent_from_origin(origin: Self::O) -> Self::E {
        origin.into()
    }

    fn with_extent(extent: Self::E) -> Self {
        (Default::default(), extent)
    }

    fn from_raw_parts(origin: Self::O, extent: Self::E) -> Self {
        (origin, extent)
    }

    fn is_multi_layered() -> bool {
        false
    }

    fn area(&self) -> u32 {
        self.extent().area()
    }
}

// Texture3D
impl Region for (vek::Vec3<u32>, vek::Extent3<u32>) {
    type O = vek::Vec3<u32>;
    type E = vek::Extent3<u32>;

    fn unit() -> Self {
        (vek::Vec3::zero(), vek::Extent3::one())
    }

    fn origin(&self) -> Self::O {
        self.0
    }

    fn extent(&self) -> Self::E {
        self.1
    }

    fn set_origin(&mut self, origin: Self::O) {
        self.0 = origin;
    }

    fn set_extent(&mut self, extent: Self::E) {
        self.1 = extent;
    }

    fn extent_from_origin(origin: Self::O) -> Self::E {
        origin.into()
    }

    fn with_extent(extent: Self::E) -> Self {
        (Default::default(), extent)
    }
    
    fn from_raw_parts(origin: Self::O, extent: Self::E) -> Self {
        (origin, extent)
    }

    fn is_multi_layered() -> bool {
        false
    }

    fn area(&self) -> u32 {
        self.extent().area()
    }
}
