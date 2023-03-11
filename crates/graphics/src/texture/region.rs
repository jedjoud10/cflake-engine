use std::{num::NonZeroU8, ops::Add};

pub type Dimension = wgpu::TextureDimension;

// Texture dimensions traits that are simply implemented for extents
pub trait Extent: Copy + std::ops::Div<u32, Output = Self> {
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

    // Check if the extent is a power of 2 extent
    fn is_power_of_two(&self) -> bool {
        match Self::dimensionality() {
            Dimension::D1 => self.width().is_power_of_two(),
            Dimension::D2 => {
                self.width().is_power_of_two()
                    && self.height().is_power_of_two()
            }
            Dimension::D3 => {
                self.width().is_power_of_two()
                    && self.height().is_power_of_two()
                    && self.depth().is_power_of_two()
            }
        }
    }

    // Caclulate the number of mipmap levels that a texture can have
    // Returns None if the extent is a NPOT extent
    // Returns 1 if the texture only has one mip
    fn levels(&self) -> Option<NonZeroU8> {
        if !self.is_power_of_two() {
            return None;
        }

        let cur = self.reduce_max() as f32;
        let num = cur.log2().floor();
        Some(
            NonZeroU8::new(u8::try_from(num as u8 + 1).unwrap())
                .unwrap_or(NonZeroU8::new(1).unwrap()),
        )
    }

    // Calculate the dimensions of a mip map level using it's index
    // Level equal to 0 meaning that it will return the base extent
    fn mip_level_dimensions(self, level: u8) -> Self {
        self / 2u32.pow(level as u32)
    }

    // Check if an extent is larger in all axii than another one
    fn is_larger_than(self, other: Self) -> bool;

    // Get the width of the extent
    fn width(&self) -> u32;

    // Get the height of the extent
    fn height(&self) -> u32;

    // Get the depth of the extent
    fn depth(&self) -> u32;

    // Create a new extent by cloning a value for all axii
    fn broadcast(val: u32) -> Self;

    // Create a new extent using a width, height, depth
    fn new(w: u32, h: u32, d: u32) -> Self;

    // Convert the extent to Extent3D
    fn decompose(&self) -> vek::Extent3<u32> {
        vek::Extent3::new(self.width(), self.height(), self.depth())
    }

    // Get the dimensionality of the extent (1, 2, or 3)
    fn dimensionality() -> Dimension;
}

// Texture offsets traits that are simply implemented for origins
pub trait Origin: Copy + Default {
    // Get the X offset of the origin
    fn x(&self) -> u32;

    // Get the Y offset of the origin
    fn y(&self) -> u32;

    // Get the Z offset of the origin
    fn z(&self) -> u32;
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

    fn dimensionality() -> Dimension {
        Dimension::D2
    }

    fn width(&self) -> u32 {
        self.w
    }

    fn height(&self) -> u32 {
        self.h
    }

    fn depth(&self) -> u32 {
        1
    }

    fn broadcast(val: u32) -> Self {
        vek::Extent2::broadcast(val)
    }

    fn new(w: u32, h: u32, _: u32) -> Self {
        vek::Extent2::new(w, h)
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

    fn dimensionality() -> Dimension {
        Dimension::D3
    }

    fn width(&self) -> u32 {
        self.w
    }

    fn height(&self) -> u32 {
        self.h
    }

    fn depth(&self) -> u32 {
        self.d
    }

    fn broadcast(val: u32) -> Self {
        vek::Extent3::broadcast(val)
    }

    fn new(w: u32, h: u32, d: u32) -> Self {
        vek::Extent3::new(w, h, d)
    }
}

// Implementation of origin for 2D vec
impl Origin for vek::Vec2<u32> {
    fn x(&self) -> u32 {
        self.x
    }

    fn y(&self) -> u32 {
        self.y
    }

    fn z(&self) -> u32 {
        0
    }
}

// Implementation of origin for 3D vec
impl Origin for vek::Vec3<u32> {
    fn x(&self) -> u32 {
        self.x
    }

    fn y(&self) -> u32 {
        self.y
    }

    fn z(&self) -> u32 {
        self.z
    }
}

// Texture region trait that will be implemented for (origin, extent) tuples
pub trait Region: Copy {
    // Regions are defined by their origin and extents
    type O: Origin
        + Default
        + Copy
        + Add<Self::O, Output = Self::O>
        + std::fmt::Debug;
    type E: Extent
        + Copy
        + Add<Self::E, Output = Self::E>
        + PartialEq
        + std::fmt::Debug;

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

    // Check if this region is larger than another region
    // Aka if the "other" region fits within self
    fn is_larger_than(self, other: Self) -> bool {
        let e =
            other.extent() + Self::extent_from_origin(other.origin());
        self.extent().is_larger_than(e)
    }

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
