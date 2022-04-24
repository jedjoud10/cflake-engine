use std::ops::Index;

// An AABB bound
pub struct AABB {
    pub min: vek::Vec3<f32>,
    pub max: vek::Vec3<f32>,
}

impl AABB {
    // Create an AABB at a specified position and half-width scale
    pub fn new(pos: vek::Vec3<f32>, hw: vek::Vec3<f32>) -> Self {
        Self {
            min: pos - hw,
            max: pos + hw,
        }
    }
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

// Fetch the min/max vertices using an index
impl Index<usize> for AABB {
    type Output = vek::Vec3<f32>;

    fn index(&self, index: usize) -> &Self::Output {
        if index == 0 {
            &self.min
        } else if index == 1{
            &self.max
        } else {
            panic!("no")
        }
    }
}

// Trait to convert any shape to an AABB
pub trait ToAABB {
    fn aabb(&self) -> AABB;
}
