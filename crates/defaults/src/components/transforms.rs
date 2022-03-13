use world::ecs::component::Component;
// The transform component
#[derive(Component)]
pub struct Transform {
    pub position: vek::Vec3<f32>,
    pub rotation: vek::Quaternion<f32>,
    pub scale: vek::Vec3<f32>,
}

// Default transform
impl Default for Transform {
    fn default() -> Self {
        Self {
            position: vek::Vec3::zero(),
            rotation: vek::Quaternion::identity(),
            scale: vek::Vec3::one(),
        }
    }
}

impl Transform {
    // Calculate the transform matrix and return it
    pub fn transform_matrix(&self) -> vek::Mat4<f32> {
        self.position_matrix() * self.rotation_matrix() * self.scale_matrix()
    }
    // Matrix for each attribute
    pub fn position_matrix(&self) -> vek::Mat4<f32> {
        vek::Mat4::<f32>::translation_3d(self.position)
    }
    pub fn rotation_matrix(&self) -> vek::Mat4<f32> {
        self.rotation.into()
    }
    pub fn scale_matrix(&self) -> vek::Mat4<f32> {
        vek::Mat4::<f32>::scaling_3d(self.scale)
    }
    // Calculate the forward, up, and right vectors
    pub fn forward(&self) -> vek::Vec3<f32> {
        self.rotation_matrix().mul_point(-vek::Vec3::unit_z())
    }
    pub fn up(&self) -> vek::Vec3<f32> {
        self.rotation_matrix().mul_point(vek::Vec3::unit_y())
    }
    pub fn right(&self) -> vek::Vec3<f32> {
        self.rotation_matrix().mul_point(vek::Vec3::unit_x())
    }
}
