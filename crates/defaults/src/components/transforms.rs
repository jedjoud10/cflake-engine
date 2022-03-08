use world::ecs::component::Component;
// The transform component
#[derive(Component)]
pub struct Transform {
    pub position: veclib::Vector3<f32>,
    pub rotation: veclib::Quaternion<f32>,
    pub scale: veclib::Vector3<f32>,
}

// Default transform
impl Default for Transform {
    fn default() -> Self {
        Self {
            position: veclib::Vector3::ZERO,
            rotation: veclib::Quaternion::IDENTITY,
            scale: veclib::Vector3::ONE,
        }
    }
}

// Transform creation
impl Transform {
    // With position
    pub fn with_position(mut self, position: veclib::Vector3<f32>) -> Self {
        self.position = position;
        self
    }
    // With rotation
    pub fn with_rotation(mut self, rotation: veclib::Quaternion<f32>) -> Self {
        self.rotation = rotation;
        self
    }
    // With scale
    pub fn with_scale(mut self, scale: veclib::Vector3<f32>) -> Self {
        self.scale = scale;
        self
    }
}

impl Transform {
    // Calculate the transform matrix and return it
    pub fn transform_matrix(&self) -> veclib::Matrix4x4<f32> {
        self.position_matrix() * self.rotation_matrix() * self.scale_matrix()
    }
    // Matrix for each attribute
    pub fn position_matrix(&self) -> veclib::Matrix4x4<f32> {
        veclib::Matrix4x4::<f32>::from_translation(self.position)
    }
    pub fn rotation_matrix(&self) -> veclib::Matrix4x4<f32> {
        // TODO: Bruh this shit's transposed
        veclib::Matrix4x4::<f32>::from_quaternion(&self.rotation).transposed()
    }
    pub fn scale_matrix(&self) -> veclib::Matrix4x4<f32> {
        veclib::Matrix4x4::<f32>::from_scale(self.scale)
    }
    // Calculate the forward, up, and right vectors
    pub fn forward(&self) -> veclib::Vector3<f32> {
        self.rotation_matrix().mul_point(&veclib::Vector3::Z)
    }
    pub fn up(&self) -> veclib::Vector3<f32> {
        self.rotation_matrix().mul_point(&veclib::Vector3::Y)
    }
    pub fn right(&self) -> veclib::Vector3<f32> {
        self.rotation_matrix().mul_point(&veclib::Vector3::X)
    }
}
