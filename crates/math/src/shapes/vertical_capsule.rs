use crate::bounds::aabb::{ToAABB, AABB};

// A simple capsule
#[derive(Clone)]
pub struct VerticalCapsule {
    // Common
    pub center: vek::Vec3<f32>,
    pub height: f32,
    pub radius: f32,
}

impl VerticalCapsule {
    // Get the bottom and top points
    pub fn bottom(&self) -> vek::Vec3<f32> {
        self.center - vek::Vec3::unit_y() * (self.height / 2.0)
    }
    pub fn top(&self) -> vek::Vec3<f32> {
        self.center + vek::Vec3::unit_y() * (self.height / 2.0)
    }
}

impl ToAABB for VerticalCapsule {
    fn aabb(&self) -> AABB {
        // Get the two points
        let p1 = self.bottom();
        let p2 = self.top();

        // Min and max
        let min = p1 - vek::Vec3::one() * self.radius;
        let max = p2 - vek::Vec3::one() * self.radius;
        AABB { min, max }
    }
}
