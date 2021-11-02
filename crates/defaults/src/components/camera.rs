use ecs::*;
use math;
use rendering::Window;
// A simple camera component
pub struct Camera {
    pub view_matrix: veclib::Matrix4x4<f32>,
    pub projection_matrix: veclib::Matrix4x4<f32>,
    pub horizontal_fov: f32,
    pub frustum: math::Frustum,
    pub aspect_ratio: f32,
    pub clip_planes: (f32, f32), // Near, far
}

// Impl block for Camera component
impl Camera {
    // Update the projection matrix of this camera
    pub fn update_projection_matrix(&mut self, window: &Window) {
        // Turn the horizontal fov into a vertical one
        let vertical_fov: f32 = 2.0 * ((self.horizontal_fov.to_radians() / 2.0).tan() * (window.dimensions.y as f32 / window.dimensions.x as f32)).atan();
        self.projection_matrix = veclib::Matrix4x4::from_perspective(self.clip_planes.0, self.clip_planes.1, self.aspect_ratio, vertical_fov);
    }
    // Calculate the view matrix using a rotation and a position
    pub fn calculate_view_matrix(position: veclib::Vector3<f32>, rotation: veclib::Quaternion<f32>) -> veclib::Matrix4x4<f32> {
        let rotation_matrix = veclib::Matrix4x4::from_quaternion(&rotation);
        let mut forward_vector = rotation_matrix.mul_point(&veclib::Vector3::<f32>::new(0.0, 0.0, -1.0));
        forward_vector.normalize();
        let mut up_vector = rotation_matrix.mul_point(&veclib::Vector3::<f32>::new(0.0, 1.0, 0.0));
        up_vector.normalize();
        veclib::Matrix4x4::look_at(&position, &up_vector, &(forward_vector + position))
    }
    // Update the view matrix using a rotation and a position
    pub fn update_view_matrix(&mut self, position: veclib::Vector3<f32>, rotation: veclib::Quaternion<f32>) {
        self.view_matrix = Self::calculate_view_matrix(position, rotation);
    }
    // Update the frustum-culling matrix
    pub fn update_frustum_culling_matrix(&mut self) {
        // Too ez m8
        let m = self.projection_matrix * self.view_matrix;
        self.frustum = math::Frustum {
            matrix: m,
            projection_matrix: self.projection_matrix,
            inverse_matrix: m.inversed(),
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            view_matrix: veclib::Matrix4x4::default_identity(),
            projection_matrix: veclib::Matrix4x4::default_identity(),
            horizontal_fov: 90.0,
            aspect_ratio: 16.0 / 9.0,
            clip_planes: (0.3, 4000.0),
            frustum: math::Frustum::default(),
        }
    }
}

ecs::impl_component!(Camera);
