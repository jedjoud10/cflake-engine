use crate::AABB;

// A simple sphere that is represented by it's center and radius
#[derive(Clone)]
pub struct Sphere {
    // The 3D center of the sphere
    pub center: vek::Vec3<f32>,

    // The radius of the sphere
    pub radius: f32,
}

impl Sphere {
    // Create a sphere with a center and radius
    pub fn new(center: vek::Vec3<f32>, radius: f32) -> Self {
        Self { center, radius }
    }

    // Create a new unit sphere at a specific location
    pub fn new_unit(center: vek::Vec3<f32>) -> Self {
        Self {
            center,
            radius: 1.0,
        }
    }

    // Create a point (a sphere with radius of 0)
    pub fn new_point(center: vek::Vec3<f32>) -> Self {
        Self {
            center,
            radius: 0.0,
        }
    }
}

// Create an AABB from a sphere
impl Into<AABB> for Sphere {
    fn into(self) -> AABB {
        AABB::new(self.center, vek::Extent3::broadcast(self.radius))
    }
}
