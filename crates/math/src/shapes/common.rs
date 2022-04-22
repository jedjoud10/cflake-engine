use super::{Cuboid, Sphere, VerticalCapsule};
use enum_as_inner::EnumAsInner;

// Some basic shapes
#[derive(EnumAsInner, Clone)]
pub enum ShapeType {
    Cuboid(Cuboid),
    Sphere(Sphere),
}

impl ShapeType {
    // Get the center of the inner basic shape
    pub fn get_center(&self) -> &vek::Vec3<f32> {
        match self {
            ShapeType::Cuboid(cuboid) => &cuboid.center,
            ShapeType::Sphere(sphere) => &sphere.center,
        }
    }
}
