use std::ops::Add;

use super::Extent;

// Texture region trait that will be implemented for (origin, extent) tuples
pub trait Region: Copy {
    // Regions are defined by their origin and extents
    type O: Default + Copy + Add<Self::O, Output = Self::O>;
    type E: Extent + Copy + From<Self::O>;

    // Create a texel region of one singular unit (so we can store a singular texel)
    fn unit() -> Self;

    // Get the region's origin
    fn origin(&self) -> Self::O;

    // Get the region's extent
    fn extent(&self) -> Self::E;

    // Sum up the origin and extent into one value
    fn summed(&self) -> Self::E;

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
