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

// Trait to convert any shape to an AABB
pub trait ToAABB {
    fn aabb(&self) -> AABB;
}
