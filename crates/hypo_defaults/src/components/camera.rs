use hypo_ecs::*;
use hypo_math as math;
use hypo_rendering::Window;
// A simple camera component
pub struct Camera {
    pub view_matrix: veclib::Matrix4x4<f32>,
    pub projection_matrix: veclib::Matrix4x4<f32>,
    pub frustum: math::Frustum,
    pub horizontal_fov: f32,
    pub aspect_ratio: f32,
    pub clip_planes: (f32, f32), // Near, far
}

// Impl block for Camera component
impl Camera {
    // Update the projection matrix of this camera
    pub fn update_projection_matrix(&mut self, window: &Window) {
        // Turn the horizontal fov into a vertical one
        let vertical_fov: f32 = 2.0 * ((self.horizontal_fov.to_radians() / 2.0).tan() * (window.size.x as f32 / window.size.y as f32)).atan();
        self.projection_matrix = veclib::Matrix4x4::from_perspective(self.clip_planes.0, self.clip_planes.1, self.aspect_ratio, vertical_fov);
    }
    // Update the view matrix using a rotation and a position
    pub fn update_view_matrix(&mut self, position: veclib::Vector3<f32>, mut rotation: veclib::Quaternion<f32>) {
        let rotation_matrix = veclib::Matrix4x4::from_quaternion(&rotation);
        let mut forward_vector = rotation_matrix.mul_point(&veclib::Vector3::<f32>::new(0.0, 0.0, -1.0));
        forward_vector.normalize();
        let mut up_vector = rotation_matrix.mul_point(&veclib::Vector3::<f32>::new(0.0, 1.0, 0.0));
        up_vector.normalize();
        self.view_matrix = veclib::Matrix4x4::from_translation(veclib::Vector3::new(4.0, 5.0, 6.0));
        //println!("{:?}", forward_vector);
        self.view_matrix = veclib::Matrix4x4::look_at(&position, &up_vector, &(forward_vector + position));
        //self.view_matrix = veclib::Matrix4x4::look_at(&veclib::Vector3::<f32>::new(5.0, 5.0, 0.0), &veclib::Vector3::default_y(), &veclib::Vector3::ZERO);
        //self.view_matrix = veclib::Matrix4x4::default_identity();
    }
    // Update the frustum-culling matrix
    pub fn update_frustum_culling_matrix(&mut self) {
        // Too ez m8
        self.frustum.matrix = self.projection_matrix * self.view_matrix;
        self.frustum.projection_matrix = self.projection_matrix;
    }
}

impl ComponentInternal for Camera {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl ComponentID for Camera {
    fn get_component_name() -> String {
        String::from("Camera Component")
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            view_matrix: veclib::Matrix4x4::default_identity(),
            projection_matrix: veclib::Matrix4x4::default_identity(),
            frustum: math::Frustum::default(),
            horizontal_fov: 90.0,
            aspect_ratio: 16.0 / 9.0,
            clip_planes: (3.0, 10000.0),
        }
    }
}

impl Component for Camera {}
