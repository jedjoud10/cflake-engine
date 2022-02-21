use super::bounds::aabb::*;
use crate::{
    octrees::{Octree, OctreeNode},
    shapes::{BasicShapeType, Sphere},
};

/* #region AABB stuff */
// Check if an AABB intersects another AABB
pub fn aabb_aabb(aabb: &AABB, other: &AABB) -> bool {
    aabb.min.elem_lt(&other.max).all() && other.min.elem_lt(&aabb.max).all()
}
// Check if a point is inside an AABB
pub fn point_aabb(point: &veclib::Vector3<f32>, aabb: &AABB) -> bool {
    aabb.min.elem_lt(point).all() && aabb.max.elem_gt(point).all()
}
// Check if an AABB is intersecting a sphere
pub fn aabb_sphere(aabb: &AABB, sphere: &Sphere) -> bool {
    let nearest_point = aabb.get_nearest_point(&sphere.center);
    point_sphere(&nearest_point, sphere)
}
/* #endregion */
/* #region Main */
// Check if a point is inside a sphere
pub fn point_sphere(point: &veclib::Vector3<f32>, sphere: &Sphere) -> bool {
    point.distance(sphere.center) < sphere.radius
}
// Check if a basic shape intersects an octree node
pub fn basic_shape_octree_node(shape: &BasicShapeType, node: &OctreeNode) -> bool {
    let aabb = node.get_aabb();
    match shape {
        BasicShapeType::Cube(cube) => aabb_aabb(&AABB::from(cube.clone()), &aabb),
        BasicShapeType::Sphere(sphere) => aabb_sphere(&aabb, sphere),
    }
}
//
/* #endregion */
/* #region Octree */
// Check if some shapes intersect an octree, and if they do, return the node indices for the nodes that intersect the shapes
pub fn shapes_octree<'a>(shapes: &[BasicShapeType], octree: &'a Octree) -> Vec<&'a OctreeNode> {
    // Loop through each octree node recursively and check collision
    let mut intersected_nodes: Vec<&'a OctreeNode> = Vec::new();
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
/* #endregion */
