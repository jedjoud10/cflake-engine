use ecs::Component;

// A transform is a combination of a position, rotaion, and scale, all stored withint a single component
#[derive(Component, Clone)]
pub struct Transform {
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

impl From<vek::Quaternion<f32>> for Transform {
    fn from(quat: vek::Quaternion<f32>) -> Self {
        Self { rotation: quat, ..Default::default() }
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

impl Transform {
    // Calculate the whole object matrix and return it
    pub fn matrix(&self) -> vek::Mat4<f32> {
        self.translation() * self.rotation() * self.scaling()
    }

    // Calculate the translation matrix
    pub fn translation(&self) -> vek::Mat4<f32> {
        vek::Mat4::<f32>::translation_3d(self.position)
    }

    // Calculate the rotation matrix
    pub fn rotation(&self) -> vek::Mat4<f32> {
        self.rotation.into()
    }

    // Calculate the scale matrix
    pub fn scaling(&self) -> vek::Mat4<f32> {
        vek::Mat4::<f32>::scaling_3d(self.scale)
    }

    // Calculate the forward vector (-Z)
    pub fn forward(&self) -> vek::Vec3<f32> {
        self.rotation().mul_point(-vek::Vec3::unit_z())
    }

    // Calculate the up vector (+Y)
    pub fn up(&self) -> vek::Vec3<f32> {
        self.rotation().mul_point(vek::Vec3::unit_y())
    }

    // Calculate the right vector (+X)
    pub fn right(&self) -> vek::Vec3<f32> {
        self.rotation().mul_point(vek::Vec3::unit_x())
    }

    // Construct a transform using an X rotation (radians)
    pub fn rotation_x(angle_radians: f32) -> Self {
        Self {
            rotation: vek::Quaternion::rotation_x(angle_radians),
            ..Default::default()
        }
    }

    // Construct a transform using a Y rotation (radians)
    pub fn rotation_y(angle_radians: f32) -> Self {
        Self {
            rotation: vek::Quaternion::rotation_y(angle_radians),
            ..Default::default()
        }
    }

    // Construct a transform using a Z rotation (radians)
    pub fn rotation_z(angle_radians: f32) -> Self {
        Self {
            rotation: vek::Quaternion::rotation_z(angle_radians),
            ..Default::default()
        }
    }

    // Construct a transform using an X width
    pub fn scale_x(width: f32) -> Self {
        Self {
            scale: vek::Vec3::new(width, 1.0, 1.0),
            ..Default::default()
        }
    }

    // Construct a transform using a Y height
    pub fn scale_y(height: f32) -> Self {
        Self {
            scale: vek::Vec3::new(1.0, height, 1.0),
            ..Default::default()
        }
    }

    // Construct a transform using a Z depth
    pub fn scale_z(depth: f32) -> Self {
        Self {
            scale: vek::Vec3::new(1.0, 1.0, depth),
            ..Default::default()
        }
    }

    // Construct a transform at the given X unit position
    pub fn at_x(x: f32) -> Self {
        Self {
            position: vek::Vec3::new(x, 0.0, 0.0),
            ..Default::default()
        }
    }

    // Construct a transform at the given Y unit position
    pub fn at_y(y: f32) -> Self {
        Self {
            position: vek::Vec3::new(0.0, y, 0.0),
            ..Default::default()
        }
    }

    // Construct a transform at the given Z unit position
    pub fn at_z(z: f32) -> Self {
        Self {
            position: vek::Vec3::new(0.0, 0.0, z),
            ..Default::default()
        }
    }

    // Construct a transform at the given X, Y, Z position
    pub fn at_xyz(x: f32, y: f32, z: f32) -> Self {
        Self::from((x, y, z))
    }
}
