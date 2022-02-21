use crate::shapes::Cube;

// An AABB bound
pub struct AABB {
    pub min: veclib::Vector3<f32>,
    pub max: veclib::Vector3<f32>,
}

// Default AABB, just a unit cube with a center at 0,0,0
impl Default for AABB {
    fn default() -> Self {
        Self {
            min: (veclib::Vector3::ONE / 2.0) - 1.0,
            max: (veclib::Vector3::ONE / 2.0),
        }
    }
}

impl AABB {
    // Get the closest point of the AABB from a specific point
    pub fn get_nearest_point(&self, point: &veclib::Vector3<f32>) -> veclib::Vector3<f32> {
        point.clamp(self.min, self.max)
    }
}

// Conversions
impl From<Cube> for AABB {
    fn from(cube: Cube) -> Self {
        let half_extent = cube.size / 2.0;
        Self {
            min: cube.center - half_extent,
            max: cube.center + half_extent,
        }
    }
}
