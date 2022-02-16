use super::bounds;
use crate::shapes::Sphere;

// Intersection tests
/* #region AABB stuff */
// Check if an AABB intersects another AABB
pub fn aabb_aabb(aabb: &bounds::AABB, other: &bounds::AABB) -> bool {
    aabb.min.elem_lt(&other.max).all() && other.min.elem_lt(&aabb.max).all()
}
// Check if a point is inside an AABB
pub fn point_aabb(point: &veclib::Vector3<f32>, aabb: &bounds::AABB) -> bool {
    aabb.min.elem_lt(point).all() && aabb.max.elem_gt(point).all()
}
// Check if an AABB is intersecting a sphere
pub fn aabb_sphere(aabb: &bounds::AABB, sphere: &Sphere) -> bool {
    let closest_point = aabb.get_nearest_point(&sphere.center);
    point_sphere(&closest_point, sphere)
}
/* #endregion */
/* #region Others */
// Check if a point is inside a sphere
pub fn point_sphere(point: &veclib::Vector3<f32>, sphere: &Sphere) -> bool {
    point.distance(sphere.center) < sphere.radius
}
/* #endregion */
