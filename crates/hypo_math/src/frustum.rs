// A view frustum
#[derive(Default, Clone)]
pub struct Frustum {
    pub matrix: veclib::Matrix4x4<f32>,
    pub projection_matrix: veclib::Matrix4x4<f32>,
}