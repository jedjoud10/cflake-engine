use crate::{Aabb, Sphere};
use std::cmp::PartialOrd;
use vek::{num_traits::real::Real, Clamp};
/*
// TODO: Halp idk where to put this
// Struct that contains collision data
pub struct Collision {
    pub normal: vek::Vec3<f32>,
    pub depth: f32,
}

// Trait implemented for two objects that we can check collisions for
// This will return a collision type that contains the normal and depth of the collision
// Should only be implemented for convex types
pub trait CollideTwo<A, B> {
    // Simpler check that just checks if they intersect
    fn intersects(a: &A, b: &B) -> bool;

    // Check if the two objects collide, and return normal information
    fn collides(a: &A, b: &B) -> Option<Collision>;
}

// Collision checker between two types
pub struct TwoColliders<A, B> {
    pub first: A,
    pub second: B,
}

// Flips A and B if it's asymmetric
pub trait Asymmetric {}
impl<A, B> CollideTwo<A, B> for TwoColliders<A, B> where TwoColliders<B, A>: CollideTwo<B, A> + Asymmetric {
    fn collides(a: &A, b: &B) -> Option<Collision> {
        <TwoColliders::<B, A> as CollideTwo<B, A>>::collides(b, a)
    }

    fn intersects(a: &A, b: &B) -> bool {
        <TwoColliders::<B, A> as CollideTwo<B, A>>::intersects(b, a)
    }
}

// Check if an AABB intersects another AABB
impl<T: PartialOrd> CollideTwo<Aabb<T>, Aabb<T>> for TwoColliders<Aabb<T>, Aabb<T>> {
    fn check(a: &Aabb<T>, b: &Aabb<T>) -> Option<Collision> {
        todo!()
    }
}

// Check if
impl<T: PartialOrd> CollideTwo<vek::Vec3<f32>, Aabb<T>> for TwoColliders<vek::Vec3<f32>, Aabb<T>> {
    fn check(a: &vek::Vec3<f32>, b: &Aabb<T>) -> Option<Collision> {
        todo!()
    }
}

impl<T: PartialOrd> Asymmetric for TwoColliders<vek::Vec3<f32>, Aabb<T>> {}
*/

pub fn aabb_aabb<T>(aabb: &Aabb<T>, other: &Aabb<T>) -> bool
where
    T: PartialOrd,
{
    let max = aabb.min.partial_cmple(&other.max).reduce_and();
    let min = other.min.partial_cmplt(&aabb.max).reduce_and();
    max && min
}

// Check if a point is inside an AABB
pub fn point_aabb<T>(point: &vek::Vec3<T>, aabb: &Aabb<T>) -> bool
where
    T: PartialOrd,
{
    aabb.min.partial_cmple(point).reduce_and() && aabb.max.partial_cmpge(point).reduce_and()
}

// Check if an AABB is intersecting a sphere
pub fn aabb_sphere<T>(aabb: &Aabb<T>, sphere: &Sphere<T>) -> bool
where
    T: PartialOrd + Real,
    vek::Vec3<T>: vek::ops::Clamp,
{
    let nearest_point = sphere.center.clamped(aabb.min, aabb.max);
    point_sphere(&nearest_point, sphere)
}

// Check if a sphere is intersecting a sphere
pub fn sphere_sphere<T>(first: &Sphere<T>, second: &Sphere<T>) -> bool
where
    T: Real,
{
    vek::Vec3::distance(second.center, second.center) < (first.radius + second.radius)
}

// Check if a point is inside a sphere
pub fn point_sphere<T>(point: &vek::Vec3<T>, sphere: &Sphere<T>) -> bool
where
    T: Real,
{
    point.distance(sphere.center) < sphere.radius
}
