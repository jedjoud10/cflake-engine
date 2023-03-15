use arrayvec::ArrayVec;
use ecs::{Component, Position, Rotation};

// A perspective camera component that will be used to render the main scene
// The camera entity does not *need* to have a transform to render, since we can set the matrices directly
#[derive(Component, Clone)]
pub struct Camera {
    // The view matrix handles translation and rotation
    view: vek::Mat4<f32>,

    // The projection matrix handles the perspective projection
    projection: vek::Mat4<f32>,

    // Horizontal field of view of the camera
    pub hfov: f32,

    // Aspect aspect_ratio of the camera
    aspect_ratio: f32,

    // Near and far clip planes
    near: f32,
    far: f32,
}

// Convert a horizontal FOV to a vertical FOV (this returns the FOV in radians)
fn horizontal_to_vertical(hfov: f32, aspect_ratio: f32) -> f32 {
    2.0 * ((hfov.to_radians() / 2.0).tan() * (1.0 / (aspect_ratio)))
        .atan()
}

// Create a new projection matrix using a ratio, a field of view, and the clip planes
fn new_projection_matrix(
    hfov: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32,
) -> vek::Mat4<f32> {
    let vfov = horizontal_to_vertical(hfov, aspect_ratio);
    vek::Mat4::<f32>::perspective_rh_zo(vfov, aspect_ratio, near, far)
}

// Create a new view matrix using a position and rotation
fn new_view_matrix(
    position: &Position,
    rotation: &Rotation,
) -> vek::Mat4<f32> {
    vek::Mat4::<f32>::look_at_rh(
        **position,
        rotation.forward() + **position,
        rotation.up(),
    )
}

impl Camera {
    // Create a new camera with it's horizontal fov, the clip planes, and an aspect ratio
    pub fn new(
        hfov: f32,
        near_plane: f32,
        far_plane: f32,
        aspect_ratio: f32,
    ) -> Self {
        Self {
            view: vek::Mat4::identity(),
            projection: vek::Mat4::identity(),
            hfov,
            near: near_plane,
            far: far_plane,
            aspect_ratio,
        }
    }

    // Update the view matrix using a position and rotation
    pub fn update_view(
        &mut self,
        position: &Position,
        rotation: &Rotation,
    ) {
        self.view = new_view_matrix(position, rotation);
    }

    // Update the projection matrix using the currently stored values
    pub fn update_projection(&mut self) {
        self.projection = new_projection_matrix(
            self.hfov,
            self.aspect_ratio,
            self.near,
            self.far,
        );
    }

    // Update the inner matrices (view & projection) using a position and rotation
    pub fn update(
        &mut self,
        position: &Position,
        rotation: &Rotation,
    ) {
        self.update_view(position, rotation);
        self.update_projection()
    }

    // Get the view matrix
    pub fn view_matrix(&self) -> &vek::Mat4<f32> {
        &self.view
    }

    // Get the projection matrix
    pub fn projection_matrix(&self) -> &vek::Mat4<f32> {
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
    pub fn set_clip_planes(
        &mut self,
        near_plane: f32,
        far_plane: f32,
    ) {
        self.near = near_plane;
        self.far = far_plane;
        self.update_projection();
    }

    // Get the view frustum planes from this camera
    pub fn frustum(&self) -> Frustum {
        let columns = (*self.projection_matrix()
            * *self.view_matrix())
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
        Self::new(90.0, 0.3, 1000.0, 16.0 / 9.0)
    }
}
