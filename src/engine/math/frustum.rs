use glam::{Vec3Swizzles, Vec4Swizzles};

use crate::engine::debug::{DebugRendererable, DebugRendererType};

// A view frustum
#[derive(Default)]
pub struct Frustum {
    pub matrix: glam::Mat4,
    pub projection_matrix: glam::Mat4
}

// Code
impl Frustum {
    // Intersection code to check if a point is inside the frustum
    pub fn is_point_inside_frustum(&self, point: glam::Vec3) -> bool {
        // An multiplication factor just to debug the frustum culling
        const factor: f32 = 1.3;

        // This automatically does the projection division for us
        let transformed_corner = self.matrix.project_point3(point);
        let transformed_ss = transformed_corner.xy();
        // Check if the point is in front of us
        if transformed_corner.z > 0.0 {
            // Check if is inside the bounds of the 2D screenspace NDC
            let min = (transformed_ss * factor).cmplt(glam::Vec2::ONE).all();
            let max = (transformed_ss * factor).cmpgt(-glam::Vec2::ONE).all();
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
        // Extract the near / far planes from the projection matrix
        //let near = self.projection_matrix.row(index)[14] / (self.projection_matrix[10] - 1.0); 
        // Project each corner of the unit cube by the frustum's matrix
        for corner in corners.iter() {
            let new_corner = *corner * 2.0 - 1.0;
            let projected_corner = self.matrix.inverse().project_point3(new_corner);
            projected_corners.push(projected_corner);
        }  
        return DebugRendererType::CUBE(projected_corners);
    }
}