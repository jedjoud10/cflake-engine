mod cuboid;
mod sphere;
pub use cuboid::*;
pub use sphere::*;

use crate::aabb::AABB;

// This trait will be implemented for well... shapes
// Shapes are defined by their geometry
// Each shape should be able to convert into an AABB
pub trait Shape: Into<AABB> {
    // Get the center position of the shape
    fn center(&self) -> vek::Vec3<f32>;
}
