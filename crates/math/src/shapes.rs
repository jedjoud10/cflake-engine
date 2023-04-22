mod cuboid;
mod sphere;

pub use cuboid::*;
use num_traits::real::Real;
pub use sphere::*;

// A shape is a 3D geometrical object that takes space
// For simplicity's sake, shapes can only be composed of real numbers, although I am going to remove this restriction later on

// TODO: Figure out if we should use the trait system or stay consistent with vek and use macros
// currently we use macros but idk if that's a good idea
pub trait Shape<T: Real>: Movable<T> + Boundable<T> + Volume<T> + SurfaceArea<T> {}

// Shapes that have a concrete positions
pub trait Movable<T: Real> {
    fn center(&self) -> vek::Vec3<T>;
    fn set_center(&mut self, new: vek::Vec3<T>);
}

// Implemented for shapes that have sharp points / corners
pub trait ExplicitVertices<T: Real> {
    type Points: 'static + Clone;
    fn points(&self) -> Self::Points;
}

// Implemented for shapes that have implicit points / corners
pub trait ImplicitVertices<T: Real> {
    type Points: 'static + Clone;
    type Settings: 'static;
    fn points(&self, settings: Self::Settings) -> Self::Points;
}

// Auto implement implicit for explicit
impl<T: Real, SV: ExplicitVertices<T>> ImplicitVertices<T> for SV {
    type Points = <SV as ExplicitVertices<T>>::Points;
    type Settings = ();

    fn points(&self, _: Self::Settings) -> Self::Points {
        <SV as ExplicitVertices<T>>::points(self)
    }
}

// Implemented for shapes that have concrete bounds
pub trait Boundable<T: Real> {
    fn bounds(&self) -> crate::Aabb<T>;
    fn scale_by(&mut self, scale: T);
    fn expand_by(&mut self, units: T);
}

// Implemented for shapes that can calculate their own volume
pub trait Volume<T: Real> {
    fn volume(&self) -> T;
}

// Implemented for shapes that can calculate their own surface area
pub trait SurfaceArea<T: Real> {
    fn area(&self) -> T;
}
