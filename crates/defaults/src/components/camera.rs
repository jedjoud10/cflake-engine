use world::ecs::component::Component;
use world::rendering::utils::DEFAULT_WINDOW_SIZE;
// A simple camera component
#[derive(Component)]
pub struct Camera {
    pub viewm: veclib::Matrix4x4<f32>,
    pub projm: veclib::Matrix4x4<f32>,
    pub horizontal_fov: f32,
    pub clip_planes: veclib::Vector2<f32>, // Near, far
}

// Impl block for Camera component
impl Camera {
    // Create a new camera with a specified FOV and clip planes
    pub fn new(fov: f32, clipn: f32, clipf: f32) -> Self {
        let mut camera = Self {
            viewm: veclib::Matrix4x4::IDENTITY,
            projm: veclib::Matrix4x4::IDENTITY,
            horizontal_fov: fov,
            clip_planes: veclib::Vector2::new(clipn, clipf),
        };
        camera.update_projection_matrix(DEFAULT_WINDOW_SIZE.x as f32 / DEFAULT_WINDOW_SIZE.y as f32);
        camera
    }
    // Update the projection matrix of this camera
    pub fn update_projection_matrix(&mut self, aspect_ratio: f32) {
        // Turn the horizontal fov into a vertical one
        let vertical_fov: f32 = 2.0 * ((self.horizontal_fov.to_radians() / 2.0).tan() * (1.0 / (aspect_ratio))).atan();
        self.projm = veclib::Matrix4x4::<f32>::from_perspective(self.clip_planes.x, self.clip_planes.y, aspect_ratio, vertical_fov);
    }
    // Calculate the view matrix using a rotation and a position
    pub fn calculate_view_matrix(position: veclib::Vector3<f32>, rotation: veclib::Quaternion<f32>) -> veclib::Matrix4x4<f32> {
        let rotation_matrix = veclib::Matrix4x4::<f32>::from_quaternion(&rotation);
        let mut forward_vector = rotation_matrix.mul_point(&veclib::Vector3::<f32>::new(0.0, 0.0, -1.0));
        forward_vector.normalize();
        let mut up_vector = rotation_matrix.mul_point(&veclib::Vector3::<f32>::new(0.0, 1.0, 0.0));
        up_vector.normalize();
        veclib::Matrix4x4::<f32>::look_at(&position, &up_vector, &(forward_vector + position))
    }
    // Update the view matrix using a rotation and a position
    pub fn update_view_matrix(&mut self, position: veclib::Vector3<f32>, rotation: veclib::Quaternion<f32>) {
        self.viewm = Self::calculate_view_matrix(position, rotation);
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(90.0, 0.3, 1000.0)
    }
}
