use std::ops::{Deref, DerefMut};

use enum_as_inner::EnumAsInner;

use super::{Cube, Sphere};

// A shape trait that shares some common methods between different shapes
pub trait Shapeable {
    // Get the center of the shape
    fn get_center(&self) -> veclib::Vector3<f32>;
}

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

// Some basic shapes
#[derive(EnumAsInner, Clone)]
pub enum BasicShapeType {
    Cube(Cube),
    Sphere(Sphere),
}