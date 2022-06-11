use crate::{bounds::aabb::AABB, Shape};

// A 3D cuboid that is defined by it's center and it's extent
#[derive(Clone)]
pub struct Cuboid {
    // The 3D center of the cuboid
    pub center: vek::Vec3<f32>,

    // The full-extent of the cuboid
    pub extent: vek::Extent3<f32>,
}

impl Cuboid {
    // Create a new cuboid using a center and it's extent
    pub fn new(center: vek::Vec3<f32>, size: vek::Extent3<f32>) -> Self {
        Self {
            center,
            extent: size,
        }
    }

    // Create a new cuboid using a center and a half-extent
    pub fn from_half_extent(center: vek::Vec3<f32>, half_extent: vek::Extent3<f32>) -> Self {
        Self {
            center,
            extent: half_extent * 2.0,
        }
    }
}

impl Shape for Cuboid {
    fn center(&self) -> vek::Vec3<f32> {
        self.center
    }
}

impl Into<AABB> for Cuboid {
    fn into(self) -> AABB {
        let half_extent = vek::Vec3::<f32>::from(self.extent) / 2.0;
        AABB {
            min: self.center - half_extent,
            max: self.center + half_extent,
        }
    }
}
