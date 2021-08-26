use std::default;

use glam::Vec3Swizzles;

use super::{bounds, shapes, Frustum};

// Intersection tests
pub struct Intersection {}

// The actual intersection tests
impl Intersection {
    // Check if an AABB intersects another AABB
    pub fn aabb_aabb(aabb: &bounds::AABB, other: &bounds::AABB) -> bool {
        return aabb.min.cmple(other.max).all() && other.min.cmple(aabb.max).all();
    }
    // Check if a point is inside a sphere
    pub fn point_sphere(point: &glam::Vec3, sphere: &shapes::Sphere) -> bool {
        return point.distance(sphere.center) < sphere.radius;
    }
    // Check if a point is inside an AABB
    pub fn point_aabb(point: &glam::Vec3, aabb: &bounds::AABB) -> bool {
        return aabb.min.cmplt(*point).all() && aabb.max.cmpgt(*point).all();
    }
    // Check if an AABB is intersecting a sphere
    pub fn aabb_sphere(aabb: &bounds::AABB, sphere: &shapes::Sphere) -> bool {
        let closest_point = aabb.get_nearest_point(&sphere.center);
        return Self::point_sphere(&closest_point, sphere);
    }
    // Check if a square intersects another square
    pub fn square_square(square: &shapes::Square, other: &shapes::Square) -> bool {
        return square.min.cmple(other.max).all() && other.min.cmple(square.max).all();
    }
    // Check if a screen space point is inside the NDC
    pub fn ss_point_limits(point: &glam::Vec2) -> bool {
        let min = (point).cmplt(glam::Vec2::ONE).all();
        let max = (point).cmpgt(-glam::Vec2::ONE).all();
        min && max
    }
    // Intersection code to check if a line intersects the frustum
    pub fn frustum_line(frustum: &Frustum, line: &shapes::Line) -> bool {
        false
    }
    // Check if an AABB intersects the camera's view frustum. Exit at the first valid intersection
    pub fn frustum_aabb(frustum: &Frustum, aabb: &bounds::AABB) -> bool {
        // TODO
        true
    }
}
