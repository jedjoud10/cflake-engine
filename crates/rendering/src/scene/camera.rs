use ecs::Component;
use math::Transform;

// A perspective camera component that will be used to render the main scene
// The camera entity does not *need* to have a transform to render, since we can set the matrices directly
#[derive(Component, Clone)]
pub struct Camera {
    // The view matrix handles translation and rotation
    view: vek::Mat4<f32>,

    // The projection matrix handles the perspective projection
    projection: vek::Mat4<f32>,

    // Horizontal field of view of the camera
    hfov: f32,

    // Aspect aspect_ratio of the camera
    aspect_ratio: f32,

    // Near and far clip planes
    near: f32,
    far: f32,
}

// Convert a horizontal FOV to a vertical FOV
fn horizontal_to_vertical(hfov: f32, aspect_ratio: f32) -> f32 {
    2.0 * ((hfov.to_radians() / 2.0).tan() * (1.0 / (aspect_ratio))).atan()
}

// Create a new projection matrix using a ratio, a field of view, and the clip planes
fn new_projection_matrix(hfov: f32, aspect_ratio: f32, near: f32, far: f32) -> vek::Mat4<f32> {
    //vek::Mat4::<f32>::perspective_fov_rh_no(horizontal_to_vertical(hfov, aspect_ratio), width, height, self.clips.x, self.clips.y);
    let verticla_fov_radians = horizontal_to_vertical(hfov, aspect_ratio).to_radians();
    vek::Mat4::<f32>::perspective_rh_no(verticla_fov_radians, aspect_ratio, near, far)
}

// Create a new view matrix using a transform
fn new_view_matrix(transform: &Transform) -> vek::Mat4<f32> {
    vek::Mat4::<f32>::look_at_rh(
        transform.position,
        transform.forward() + transform.position,
        transform.up(),
    )
}

impl Camera {
    // Create a new camera with it's horizontal fov, the clip planes, and an aspect ratio
    pub fn new(hfov: f32, near_plane: f32, far_plane: f32, aspect_ratio: f32) -> Self {
        Self {
            view: vek::Mat4::identity(),
            projection: vek::Mat4::identity(),
            hfov,
            near: near_plane,
            far: far_plane,
            aspect_ratio,
        }
    }

    // Update the view matrix using a transform
    pub fn update_view(&mut self, transform: &Transform) {
        self.view = new_view_matrix(transform);
    }

    // Update the projection matrix using the currently stored values
    pub fn update_projection(&mut self) {
        self.projection = new_projection_matrix(self.hfov, self.aspect_ratio, self.near, self.far);
    }

    // Update the inner matrices (view & projection) using a transform
    pub fn update(&mut self, transform: &Transform) {
        self.update_view(transform);
        self.update_projection()
    }

    // Get the view matrix
    pub fn view(&self) -> &vek::Mat4<f32> {
        &self.view
    }

    // Get the projection matrix
    pub fn projection(&self) -> &vek::Mat4<f32> {
        &self.projection
    }

    // Get the horizontal FOV
    pub fn hfov(&self) -> f32 {
        self.hfov
    }

    // Get the near & far planes
    pub fn clip_planes(&self) -> vek::Vec2<f32> {
        vek::Vec2::new(self.near, self.far)
    }

    // Set the view matrix
    pub fn set_view(&mut self, view: vek::Mat4<f32>) {
        self.view = view;
    }

    // Set the projection matrix
    pub fn set_projection(&mut self, projection: vek::Mat4<f32>) {
        self.projection = projection;
    }

    // Set the horizontal fov
    pub fn set_hfov(&mut self, hfov: f32) {
        self.hfov = hfov;
        self.update_projection();
    }

    // Set the aspect ratio
    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
        self.update_projection();
    }

    // Set the clip planes
    pub fn set_clip_planes(&mut self, near_plane: f32, far_plane: f32) {
        self.near = near_plane;
        self.far = far_plane;
        self.update_projection();
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(90.0, 0.3, 1000.0, 16.0 / 9.0)
    }
}
