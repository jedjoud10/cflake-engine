use arrayvec::ArrayVec;
use ecs::{Component, Position, Rotation};
use math::Frustum;

// A perspective camera component that will be used to render the main scene
// The camera entity does not *need* to have a transform to render, since we can set the matrices directly
#[derive(Component, Clone, Copy)]
pub struct Camera {
    // Horizontal field of view of the camera
    pub hfov: f32,

    // Aspect aspect_ratio of the camera
    pub aspect_ratio: f32,

    // Near and far clip planes
    pub near: f32,
    pub far: f32,
}

// Convert a horizontal FOV to a vertical FOV (this returns the FOV in radians)
pub fn horizontal_to_vertical(hfov: f32, ratio: f32) -> f32 {
    2.0 * ((hfov.to_radians() / 2.0).tan() * (1.0 / (ratio))).atan()
}

impl Camera {
    // Create a new view matrix using a position and rotation
    pub fn view_matrix(
        &self,
        position: &Position,
        rotation: &Rotation,
    ) -> vek::Mat4<f32> {
        vek::Mat4::<f32>::look_at_rh(
            **position,
            rotation.forward() + **position,
            rotation.up(),
        )
    }

    // Create a new projection matrix using thec camera's parameters
    pub fn projection_matrix(&self) -> vek::Mat4<f32> {
        let hfov = self.hfov.clamp(0.01, 179.99);
        let ratio = self.aspect_ratio.max(0.01);
        let near = self.near.max(0.0001);
        let far = self.far.max(near);
        let vfov = horizontal_to_vertical(hfov, ratio);
        vek::Mat4::<f32>::perspective_rh_zo(vfov, ratio, near, far)
    }

    // Get the view frustum planes from this camera
    pub fn frustum(
        &self,
        position: &Position,
        rotation: &Rotation,
    ) -> Frustum<f32> {
        let view = self.view_matrix(position, rotation);
        let projection = self.projection_matrix();
        Frustum::<f32>::from_camera_matrices(projection, view)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            hfov: 120.0,
            aspect_ratio: 16.0 / 9.0,
            near: 0.01,
            far: 5000.0,
        }
    }
}
