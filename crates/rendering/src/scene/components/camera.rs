use arrayvec::ArrayVec;
use ecs::{Component, Position, Rotation};

// A perspective camera component that will be used to render the main scene
// The camera entity does not *need* to have a transform to render, since we can set the matrices directly
#[derive(Component, Clone)]
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
    ) -> Frustum {
        let columns = (self.projection_matrix()
            * self.view_matrix(position, rotation))
        .transposed()
        .into_col_arrays();
        let columns = columns
            .into_iter()
            .map(vek::Vec4::from)
            .collect::<ArrayVec<vek::Vec4<f32>, 4>>();

        // Magic from https://www.braynzarsoft.net/viewtutorial/q16390-34-aabb-cpu-side-frustum-culling
        // And also from https://gamedev.stackexchange.com/questions/156743/finding-the-normals-of-the-planes-of-a-view-frustum
        // YAY https://stackoverflow.com/questions/12836967/extracting-view-frustum-planes-gribb-hartmann-method
        let left = FrustumPlane::new(columns[3] + columns[0]);
        let right = FrustumPlane::new(columns[3] - columns[0]);
        let top = FrustumPlane::new(columns[3] - columns[1]);
        let bottom = FrustumPlane::new(columns[3] + columns[1]);
        let near = FrustumPlane::new(columns[3] + columns[2]);
        let far = FrustumPlane::new(columns[3] - columns[2]);
        [top, bottom, left, right, near, far]
    }
}

// A whhole view frustum
pub type Frustum = [FrustumPlane; 6];

// A single frustum plane
#[derive(Clone, Copy, PartialEq)]
pub struct FrustumPlane {
    pub normal: vek::Vec3<f32>,
    pub distance: f32,
}

impl FrustumPlane {
    fn new(column: vek::Vec4<f32>) -> Self {
        let mag = column.xyz().magnitude();
        Self {
            normal: column.xyz() / mag,
            distance: (column.w / mag),
        }
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
