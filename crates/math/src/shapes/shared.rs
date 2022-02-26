use super::{Cuboid, Sphere};
use enum_as_inner::EnumAsInner;

// Some basic shapes
#[derive(EnumAsInner, Clone)]
pub enum BasicShapeType {
    Cuboid(Cuboid),
    Sphere(Sphere),
}

impl BasicShapeType {
    // Get the center of the inner basic shape
    pub fn get_center(&self) -> &veclib::Vector3<f32> {
        match self {
            BasicShapeType::Cuboid(cuboid) => &cuboid.center,
            BasicShapeType::Sphere(sphere) => &sphere.center,
        }
    }
}
