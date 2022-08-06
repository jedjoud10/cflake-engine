mod cuboid;
mod sphere;
pub use cuboid::*;
pub use sphere::*;
use crate::AABB;

// A shape is a 3D geometrical object that takes space
pub trait Shape: Movable + Boundable + Volume + SurfaceArea + Sync + Send {
}

// Shapes that have a concrete positions
pub trait Movable {
    fn center(&self) -> vek::Vec3<f32>;
    fn set_center(&mut self, new: vek::Vec3<f32>);
}

// Implemented for shapes that have sharp points / corners
pub trait SharpVertices {
    type Points: 'static + Clone;
    fn points(&self) -> Self::Points;
}

// Implemented for shapes that have concrete bounds
pub trait Boundable {
    fn bounds(&self) -> AABB;
    fn scale_by(&mut self, scale: f32);
    fn expand_by(&mut self, expand_units: f32);
}

// Implemented for shapes that can calculate their own volume 
pub trait Volume {
    fn volume(&self) -> f32;
}

// Implemented for shapes that can calculate their own surface area
pub trait SurfaceArea {
    fn surface_area(&self) -> f32;
}