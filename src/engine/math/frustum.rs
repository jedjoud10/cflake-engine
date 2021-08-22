use glam::Vec4Swizzles;

use crate::engine::debug::{DebugRendererable, DebugRendererType};

// A view frustum
#[derive(Default)]
pub struct Frustum {
    pub matrix: glam::Mat4
}

// Code
impl Frustum {
    // Intersection code to check if a point is inside the frustum
    pub fn is_point_inside_frustum(&self, point: glam::Vec3) -> bool {
        // An multiplication factor just to debug the frustum culling
        const factor: f32 = 1.3;

        let transformed_corner = self.matrix.mul_vec4(glam::vec4(point.x, point.y, point.z, 1.0));
        // You have to divide by the W scalar first to get the screenspace NDC
        let transformed_corner_screen_space = transformed_corner.xy() / transformed_corner.w;
        // Check if the point is in front of us
        if transformed_corner.z > 0.0 {
            // Check if is inside the bounds of the 2D screenspace NDC
            let min = (transformed_corner_screen_space * factor).cmplt(glam::Vec2::ONE).all();
            let max = (transformed_corner_screen_space * factor).cmpgt(-glam::Vec2::ONE).all();
            if min && max {
                // The pojnt is inside the frustum
                return true;
            } else { return false; }
        } else {
            // The projected corner was behind us, so it was not inside the frustum
            return false;
        }
    }
}

// The frustum can be debug drawed
impl DebugRendererable for Frustum {
    // Turn the frustum into a cube and render it
    fn get_debug_renderer(&self) -> DebugRendererType {
        let corners = super::shapes::CUBE_CORNERS;   
        let mut projected_corners: Vec<glam::Vec3> = Vec::new();   
        // Project each corner of the unit cube by the frustum's matrix
        for corner in corners.iter() {
            let projected_corner = self.matrix.mul_vec4(glam::vec4(corner.x, corner.y, corner.z, 1.0));
            projected_corners.push(projected_corner.xyz() / projected_corner.w);
        }  
        return DebugRendererType::CUBE(projected_corners);
    }
}