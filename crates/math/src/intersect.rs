use std::cmp::PartialOrd;

use vek::{Clamp, num_traits::real::Real};

use crate::{Sphere, Aabb};

// Check if an AABB intersects another AABB
pub fn aabb_aabb<T>(aabb: &Aabb<T>, other: &Aabb<T>) -> bool where T: PartialOrd {
    let max = aabb.min.partial_cmple(&other.max).reduce_and();
    let min = other.min.partial_cmplt(&aabb.max).reduce_and();
    max && min
}

// Check if a point is inside an AABB
pub fn point_aabb<T>(point: &vek::Vec3<T>, aabb: &Aabb<T>) -> bool where T: PartialOrd {
    aabb.min.partial_cmple(point).reduce_and()
        && aabb.max.partial_cmpgt(point).reduce_and()
}

// Check if an AABB is intersecting a sphere
pub fn aabb_sphere<T>(aabb: &Aabb<T>, sphere: &Sphere<T>) -> bool where T: PartialOrd + Real, vek::Vec3<T>: vek::ops::Clamp {
    let nearest_point = sphere.center.clamped(aabb.min, aabb.max);
    point_sphere(&nearest_point, sphere)
}

// Check if a sphere is intersecting a sphere
pub fn sphere_sphere<T>(first: &Sphere<T>, second: &Sphere<T>) -> bool where T: Real {
    vek::Vec3::distance(second.center, second.center)
        < (first.radius + second.radius)
}

// Check if a point is inside a sphere
pub fn point_sphere<T>(point: &vek::Vec3<T>, sphere: &Sphere<T>) -> bool where T: Real {
    point.distance(sphere.center) < sphere.radius
}