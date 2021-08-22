use glam::Vec3Swizzles;

use crate::engine::debug::{DebugRendererType, DebugRendererable};

use super::shapes;

// A view frustum
#[derive(Default, Clone)]
pub struct Frustum {
    pub matrix: glam::Mat4,
    pub projection_matrix: glam::Mat4,
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
        DebugRendererType::CUBE(projected_corners)
    }
}
