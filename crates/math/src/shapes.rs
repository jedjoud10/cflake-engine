mod cuboid;
mod sphere;
pub use cuboid::*;
pub use sphere::*;

use crate::AABB;

// A shape is a 3D geometrical object that takes space
// Shapes have no masses, they solely represent the geometry of certain pre-defined primitives
pub trait Shape: Clone + Sync + Send {
    // Get the center position of the shape
    fn center(&self) -> vek::Vec3<f32>;

    // Get the AABB bounds of the shape
    fn bounds(&self) -> AABB;
}
