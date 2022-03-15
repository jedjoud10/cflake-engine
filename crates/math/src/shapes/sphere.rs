use crate::bounds::aabb::ToAABB;

// A simple sphere
#[derive(Clone)]
pub struct Sphere {
    // Common
    pub center: vek::Vec3<f32>,
    pub radius: f32,
}

impl ToAABB for Sphere {
    fn aabb(&self) -> crate::bounds::aabb::AABB {
        todo!()
    }
}