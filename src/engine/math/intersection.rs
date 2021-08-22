use glam::Vec3Swizzles;

use super::{Frustum, bounds, shapes};

// Intersection tests
pub struct Intersection {}

// The actual intersection tests
impl Intersection {
    // Check if an AABB intersects a sphere
    pub fn aabb_sphere(aabb: &bounds::AABB, sphere: &shapes::Sphere) -> bool {
        false
    }
    // Check if an AABB intersects another AABB
    pub fn aabb_aabb(aabb: &bounds::AABB, other: &bounds::AABB) -> bool {
        false
    }
    // Intersection code to check if a point is inside the frustum
    pub fn frustum_point(frustum: &Frustum, point: glam::Vec3) -> bool {
        // An multiplication factor just to debug the frustum culling
        const FACTOR: f32 = 1.0;

        // This automatically does the projection division for us
        let transformed_corner = frustum.matrix.project_point3(point);
        let transformed_ss = transformed_corner.xy();
        // Check if the point is in front of us
        if transformed_corner.z < 1.0 {
            // Check if is inside the bounds of the 2D screenspace NDC
            let min = (transformed_ss * FACTOR).cmplt(glam::Vec2::ONE).all();
            let max = (transformed_ss * FACTOR).cmpgt(-glam::Vec2::ONE).all();
            min && max
        } else {
            // The projected corner was behind us, so it was not inside the frustum
            false
        }
    }
    // Intersection code to check if a line intersects the frustum
    pub fn frustum_line(frustum: &Frustum, line: &shapes::Line) -> bool {
        false
    }
    // Check if an AABB intersects the camera's view frustum. Exit at the first valid intersection
    pub fn frustum_aabb(frustum: &Frustum, aabb: &bounds::AABB) -> bool {
        // Get all the corners from this AABB and transform them by the matrix, then check if they fit inside the NDC
        for corner_index in 0..8 {
            let corner = aabb.get_corner(corner_index);
            // Check if one of the corners is inside the frustum, if it isn't just skip to the next one
            if Self::frustum_point(frustum, corner) {
                return true;
            } else {
                continue;
            }
        }
        false
    }
}
