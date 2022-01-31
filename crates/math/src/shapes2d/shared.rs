use std::ops::{Deref, DerefMut};

use super::{Square, Polygon2D};

// A shape trait that shares some common methods between different 2d shapes
pub trait Shapeable2D {}

// A main shape struct
pub struct Shape2D<T: Shapeable2D>(T);

impl<T: Shapeable2D> Deref for Shape2D<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Shapeable2D> DerefMut for Shape2D<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// An enum for 2D shapes
pub enum ShapeType2D {
    Square(Square),
    Polygon(Polygon2D),
}