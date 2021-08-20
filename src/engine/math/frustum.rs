use super::shapes::Plane;
use crate::engine::core::defaults::components::components;
use glam::Vec4Swizzles;

// A frustum
#[derive(Default)]
pub struct Frustum {
    pub clips: (f32, f32),
    pub aspect_ratio: f32,
    pub horizontal_fov: f32,
    pub planes: Vec<Plane>,
}

// Le kode
impl Frustum {
    // Calculate all the 6 planes that consist this frustum
    pub fn calculate_planes(&mut self, _camera_position: glam::Vec3, _camera_rotation: glam::Quat, camera_data: &components::Camera) -> Vec<Plane> {
        let planes: Vec<Plane> = Vec::new();
        // Create a simple cube, then transform all of it's vertices by the projection matrix and then view matrix, that should leave us with the view frustum vertices
        let mut vertices: Vec<glam::Vec3> = vec![
            glam::vec3(-1.0, 0.0, 0.0),
            glam::vec3(1.0, 0.0, 0.0),
            glam::vec3(0.0, -1.0, 0.0),
            glam::vec3(0.0, 1.0, 0.0),
            glam::vec3(0.0, 0.0, -1.0),
            glam::vec3(0.0, 0.0, 1.0),
        ];
        // Transform all the points by the view matrix
        for vertex in vertices.iter_mut() {
            let new_vertex = camera_data.projection_matrix.mul_vec4(glam::vec4(vertex.x, vertex.y, vertex.z, 1.0)).xyz();
            *vertex = new_vertex;
            println!("{}", vertex);
        }
        planes
    }
}
