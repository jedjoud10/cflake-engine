use std::ops::{Deref, DerefMut};

use super::{Cube, Sphere};

// A shape trait that shares some common methods between different shapes
pub trait Shapeable {}

// A main shape struct
pub struct Shape<T: Shapeable>(T);

impl<T: Shapeable> Deref for Shape<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Shapeable> DerefMut for Shape<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// An enum for 3D shapes
pub enum ShapeType {
    Cube(Cube),
    Sphere(Sphere),
}