use world::ecs::Component;
// The transform component
#[derive(Component, Clone)]
pub struct Transform {
    // Position, rotation, scale
    pub position: vek::Vec3<f32>,
    pub rotation: vek::Quaternion<f32>,
    pub scale: vek::Vec3<f32>,
}

impl From<vek::Transform<f32, f32, f32>> for Transform {
    fn from(t: vek::Transform<f32, f32, f32>) -> Self {
        Self {
            position: t.position,
            rotation: t.orientation,
            scale: t.scale,
        }
    }
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

impl From<vek::Vec3<f32>> for Transform {
    fn from(vec: vek::Vec3<f32>) -> Self {
        Self {
            position: vec,
            ..Default::default()
        }
    }
}

impl From<(f32, f32, f32)> for Transform {
    fn from(vec: (f32, f32, f32)) -> Self {
        Self {
            position: vec.into(),
            ..Default::default()
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
    // Rotation constructors
    pub fn rotation_x(angle_radians: f32) -> Self {
        Self {
            rotation: vek::Quaternion::rotation_x(angle_radians),
            ..Default::default()
        }
    }
    pub fn rotation_y(angle_radians: f32) -> Self {
        Self {
            rotation: vek::Quaternion::rotation_y(angle_radians),
            ..Default::default()
        }
    }
    pub fn rotation_z(angle_radians: f32) -> Self {
        Self {
            rotation: vek::Quaternion::rotation_z(angle_radians),
            ..Default::default()
        }
    }
    // Scale constructors
    pub fn scale_x(width: f32) -> Self {
        Self {
            scale: vek::Vec3::unit_x() * width,
            ..Default::default()
        }
    }
    pub fn scale_y(height: f32) -> Self {
        Self {
            scale: vek::Vec3::unit_y() * height,
            ..Default::default()
        }
    }
    pub fn scale_z(depth: f32) -> Self {
        Self {
            scale: vek::Vec3::unit_z() * depth,
            ..Default::default()
        }
    }
    // Kinda like constructor modifiers
    pub fn scaled_by(mut self, mul: vek::Vec3<f32>) -> Self {
        self.scale *= mul;
        self
    }
    pub fn offsetted_by(mut self, offset: vek::Vec3<f32>) -> Self {
        self.position += offset;
        self
    }
    pub fn rotated_by(mut self, rot: vek::Quaternion<f32>) -> Self {
        self.rotation = self.rotation * rot;
        self
    }
}
