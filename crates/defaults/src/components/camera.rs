use world::ecs::Component;
use world::rendering::utils::DEFAULT_WINDOW_SIZE;
// A simple camera component
#[derive(Component)]
pub struct Camera {
    // View matrix for translation/rotation
    pub view: vek::Mat4<f32>,

    // Pesrpective matrix for well, perspective
    pub perspective: vek::Mat4<f32>,
    
    // Other
    pub fov: f32,
    pub clips: vek::Vec2<f32>,
}

// Impl block for Camera component
impl Camera {
    // Create a new camera with a specified FOV and clip planes
    pub fn new(fov: f32, clipn: f32, clipf: f32) -> Self {
        let mut camera = Self {
            view: vek::Mat4::identity(),
            perspective: vek::Mat4::identity(),
            fov,
            clips: vek::Vec2::new(clipn, clipf),
        };
        camera.update_perspective_matrix(DEFAULT_WINDOW_SIZE.w as f32, DEFAULT_WINDOW_SIZE.h as f32);
        camera
    }
    // Update the perspective matrix of this camera
    pub fn update_perspective_matrix(&mut self, width: f32, height: f32) {
        // Calculate aspect ratio
        let ratio = width / height;
        // Turn the horizontal fov into a vertical one
        let vertical_fov: f32 = 2.0 * ((self.fov.to_radians() / 2.0).tan() * (1.0 / (ratio))).atan();
        self.perspective = vek::Mat4::<f32>::perspective_fov_rh_no(vertical_fov, width, height, self.clips.x, self.clips.y);
    }
    // Update the view matrix using a rotation and a position
    pub fn update_view_matrix(&mut self, position: vek::Vec3<f32>, forward: vek::Vec3<f32>, up: vek::Vec3<f32>) {
        // Update matrix
        self.view = vek::Mat4::<f32>::look_at_rh(position, forward + position, up);
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(90.0, 0.3, 1000.0)
    }
}
