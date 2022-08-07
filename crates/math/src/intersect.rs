use vek::Clamp;

use crate::{Sphere, AABB, Shape, Octree, Node};

// Check if an AABB intersects another AABB
pub fn aabb_aabb(aabb: &AABB, other: &AABB) -> bool {
    let max = aabb.min.partial_cmple(&other.max).reduce_and();
    let min = other.min.partial_cmplt(&aabb.max).reduce_and();
    max && min
}

// Check if a point is inside an AABB
pub fn point_aabb(point: &vek::Vec3<f32>, aabb: &AABB) -> bool {
    aabb.min.partial_cmple(point).reduce_and() && aabb.max.partial_cmpgt(point).reduce_and()
}

// Check if an AABB is intersecting a sphere
pub fn aabb_sphere(aabb: &AABB, sphere: &Sphere) -> bool {
    let nearest_point = sphere.center.clamped(aabb.min, aabb.max);
    point_sphere(&nearest_point, sphere)
}

// Check if a sphere is intersecting a sphere
pub fn sphere_sphere(first: &Sphere, second: &Sphere) -> bool {
    vek::Vec3::distance(second.center, second.center) < (first.radius + second.radius)
}

// Check if a point is inside a sphere
pub fn point_sphere(point: &vek::Vec3<f32>, sphere: &Sphere) -> bool {
    point.distance(sphere.center) < sphere.radius
}

// Check if some shapes intersect an octree, and if they do, return the node indices for the nodes that intersect the shapes
pub fn shapes_octree<'a>(shapes: &[&dyn Shape], octree: &'a Octree) -> Vec<&'a Node> {
    octree.recurse(|node| shapes.iter().map(|shape| shape.bounds()).any(|bound| aabb_aabb(&node.aabb(), &bound)))
}
