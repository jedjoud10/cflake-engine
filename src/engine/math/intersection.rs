use std::default;

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
        return aabb.min.cmple(other.max).all() && other.min.cmple(aabb.max).all();
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
            return Self::ss_point_limits(&transformed_ss);
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
        let mut square_min: glam::Vec2 = glam::Vec2::ZERO;
        let mut square_max: glam::Vec2 = glam::Vec2::ZERO;
        let mut valid_dir: bool = false;
        let mut test: bool = true;
        for corner_index in 0..8 {
            let corner = aabb.get_corner(corner_index);
            // Check if one of the corners is inside the frustum, if it isn't just skip to the next one
            if Self::frustum_point(frustum, corner) {
                return true;
            }
            let projected_point = frustum.matrix.project_point3(corner);
            valid_dir |= projected_point.z < 1.0;
            
            // Ignore the projected points that are behind us
            if projected_point.z < 1.0 {
                if test {
                    square_min = projected_point.xy();
                    square_max = projected_point.xy();
                }
                square_min = square_min.min(projected_point.xy());
                square_max = square_max.max(projected_point.xy());
                test = false;
            }
        } 
        let square = shapes::Square {
            min: square_min,
            max: square_max,
        };
        // If there where no corners on the screen, flatten them, then create a square from that and test it
        let the_questio = Self::square_square(&square, &shapes::Square {
            min: glam::vec2(-1.0, -1.0),
            max: glam::vec2(1.0, 1.0),
        });
        return the_questio && valid_dir;
    }
}
