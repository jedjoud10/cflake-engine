use super::{Cuboid, Sphere};
use enum_as_inner::EnumAsInner;

// Some basic shapes
#[derive(EnumAsInner, Clone)]
pub enum BasicShapeType {
    Cuboid(Cuboid),
    Sphere(Sphere),
}
