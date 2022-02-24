use super::{Cube, Sphere};
use enum_as_inner::EnumAsInner;

// Some basic shapes
#[derive(EnumAsInner, Clone)]
pub enum BasicShapeType {
    Cube(Cube),
    Sphere(Sphere),
}
