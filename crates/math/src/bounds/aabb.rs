use crate::shapes::Cuboid;

// An AABB bound
pub struct AABB {
    pub min: vek::Vec3<f32>,
    pub max: vek::Vec3<f32>,
}

// Default AABB, just a unit cuboid with a center at 0,0,0
impl Default for AABB {
    fn default() -> Self {
        Self {
            min: (vek::Vec3::one() / 2.0) - 1.0,
            max: (vek::Vec3::one() / 2.0),
        }
    }
}

// Conversions
impl From<Cuboid> for AABB {
    fn from(cuboid: Cuboid) -> Self {
        let half_extent = cuboid.size / 2.0;
        Self {
            min: cuboid.center - half_extent,
            max: cuboid.center + half_extent,
        }
    }
}
impl From<AABB> for Cuboid {
    fn from(aabb: AABB) -> Self {
        let full_extent = aabb.max - aabb.min;
        Self {
            center: aabb.min + (full_extent / 2.0),
            size: full_extent,
        }
    }
}
