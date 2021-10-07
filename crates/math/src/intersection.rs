use super::{bounds, shapes};
use std::default;

// Intersection tests
pub struct Intersection {}

// The actual intersection tests
impl Intersection {
    // Check if an AABB intersects another AABB
    pub fn aabb_aabb(aabb: &bounds::AABB, other: &bounds::AABB) -> bool {
        return aabb.min.elem_lte(&other.max).all() && other.min.elem_lte(&aabb.max).all();
    }
    // Check if a point is inside a sphere
    pub fn point_sphere(point: &veclib::Vector3<f32>, sphere: &shapes::Sphere) -> bool {
        return point.distance(sphere.center) < sphere.radius;
    }
    // Check if a point is inside an AABB
    pub fn point_aabb(point: &veclib::Vector3<f32>, aabb: &bounds::AABB) -> bool {
        return aabb.min.elem_lt(point).all() && aabb.max.elem_gt(point).all();
    }
    // Check if an AABB is intersecting a sphere
    pub fn aabb_sphere(aabb: &bounds::AABB, sphere: &shapes::Sphere) -> bool {
        let closest_point = aabb.get_nearest_point(&sphere.center);
        return Self::point_sphere(&closest_point, sphere);
    }
    // Check if a square intersects another square
    pub fn square_square(square: &shapes::Square, other: &shapes::Square) -> bool {
        return square.min.elem_lte(&other.max).all() && other.min.elem_lte(&square.max).all();
    }
    // Check if a screen space point is inside the NDC
    pub fn ss_point_limits(point: &veclib::Vector2<f32>) -> bool {
        let min = (point).elem_lt(&veclib::Vector2::ONE).all();
        let max = (point).elem_gt(&-veclib::Vector2::ONE).all();
        min && max
    }
    // Check if a segment intersects an aabb
    pub fn edge_aabb(segment: &shapes::Line, aabb: &bounds::AABB) -> bool {
        todo!();
    }
}
