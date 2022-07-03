use vek::Clamp;

use crate::{Sphere, AABB};

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

// Check if a point is inside a sphere
pub fn point_sphere(point: &vek::Vec3<f32>, sphere: &Sphere) -> bool {
    point.distance(sphere.center) < sphere.radius
}

/*
// Check if some shapes intersect an octree, and if they do, return the node indices for the nodes that intersect the shapes
pub fn shapes_octree<'a>(shapes: &[ShapeType], octree: &'a Octree) -> Vec<&'a Node> {
    // Loop through each octree node recursively and check collision
    let mut intersected_nodes: Vec<&'a Node> = Vec::new();
    octree.recurse(|node| {
        // Check intersections with each shape
        let mut intersects = false;
        for shape in shapes {
            intersects |= basic_shape_octree_node(shape, node);
            if intersects {
                // This node intersects one of the shapes
                intersected_nodes.push(node);
                break;
            }
        }
        intersects
    });
    intersected_nodes
}
*/
