mod cuboid;
mod sphere;
pub use cuboid::*;
pub use sphere::*;
use crate::AABB;

// A shape is a 3D geometrical object that takes space
pub trait Shape: Movable + Boundable + Volume + Area + Sync + Send {
}

// Shapes that have a concrete positions
pub trait Movable {
    fn center(&self) -> vek::Vec3<f32>;
    fn set_center(&mut self, new: vek::Vec3<f32>);
}

// Shapes that have concrete bounds
pub trait Boundable {
    fn bounds(&self) -> AABB;
    fn scale_by(&mut self, scale: f32);
    fn expand_by(&mut self, expand_units: f32);
}

// Calculate the volume of certain shapes 
pub trait Volume {
    fn volume(&self) -> f32;
}

// Calculate the surface area of certain shapes 
pub trait Area {
    fn area(&self) -> f32;
}