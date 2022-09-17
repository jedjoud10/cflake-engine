use crate::IntoMatrix;
use ecs::Component;
use std::ops::{Deref, DerefMut, Mul};

#[derive(Default, Clone, Copy, Component)]
pub struct Rotation(vek::Quaternion<f32>);

impl Rotation {
    // Calculate the forward vector (-Z)
    pub fn forward(&self) -> vek::Vec3<f32> {
        self.into_matrix().mul_point(-vek::Vec3::unit_z())
    }

    // Calculate the up vector (+Y)
    pub fn up(&self) -> vek::Vec3<f32> {
        self.into_matrix().mul_point(vek::Vec3::unit_y())
    }

    // Calculate the right vector (+X)
    pub fn right(&self) -> vek::Vec3<f32> {
        self.into_matrix().mul_point(vek::Vec3::unit_x())
    }

    // Construct a rotation using an X rotation (radians)
    pub fn rotation_x(angle_radians: f32) -> Self {
        Self(vek::Quaternion::rotation_x(angle_radians))
    }

    // Construct a rotation using a Y rotation (radians)
    pub fn rotation_y(angle_radians: f32) -> Self {
        Self(vek::Quaternion::rotation_y(angle_radians))
    }

    // Construct a rotation using a Z rotation (radians)
    pub fn rotation_z(angle_radians: f32) -> Self {
        Self(vek::Quaternion::rotation_z(angle_radians))
    }

    // Construct a rotation that is looking directly down (forward => (0, -1, 0))
    pub fn looking_down() -> Self {
        Self::rotation_x(90.0f32.to_radians())
    }

    // Construct a rotation that is looking directly up (forward => (0, 1, 0))
    pub fn looking_up() -> Self {
        Self::rotation_x(-90.0f32.to_radians())
    }

    /*
    // Construct a rotation that is looking directly right (forward => (1, 0, 0))
    pub fn looking_right() -> Self {
        Self::rotation_y(90.0f32.to_radians())
    }
    */
}

impl IntoMatrix for Rotation {
    fn into_matrix(self) -> vek::Mat4<f32> {
        self.0.into()
    }
}

impl Deref for Rotation {
    type Target = vek::Quaternion<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Rotation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<vek::Quaternion<f32>> for Rotation {
    fn as_ref(&self) -> &vek::Quaternion<f32> {
        &self.0
    }
}

impl AsMut<vek::Quaternion<f32>> for Rotation {
    fn as_mut(&mut self) -> &mut vek::Quaternion<f32> {
        &mut self.0
    }
}

impl Into<vek::Quaternion<f32>> for Rotation {
    fn into(self) -> vek::Quaternion<f32> {
        self.0
    }
}

impl From<vek::Quaternion<f32>> for Rotation {
    fn from(q: vek::Quaternion<f32>) -> Self {
        Self(q)
    }
}

impl Mul<Rotation> for Rotation {
    type Output = Rotation;

    fn mul(self, rhs: Rotation) -> Self::Output {
        Rotation(self.0 * rhs.0)
    }
}
