use crate::Component;
use std::ops::{Deref, DerefMut};

#[derive(Clone, Copy, Component)]
pub struct Scale(vek::Vec3<f32>);

impl Default for Scale {
    fn default() -> Self {
        Self(vek::Vec3::one())
    }
}

impl Scale {
    // Construct a scale using an X width
    pub fn scale_x(width: f32) -> Self {
        Self(vek::Vec3::new(width, 1.0, 1.0))
    }

    // Construct a scale using a Y height
    pub fn scale_y(height: f32) -> Self {
        Self(vek::Vec3::new(1.0, height, 1.0))
    }

    // Construct a scale using a Z depth
    pub fn scale_z(depth: f32) -> Self {
        Self(vek::Vec3::new(1.0, 1.0, depth))
    }

    // Construct a scale with it's raw data
    pub fn scale_xyz(x: f32, y: f32, z: f32) -> Self {
        Self(vek::Vec3::new(x, y, z))
    }
}


impl Deref for Scale {
    type Target = vek::Vec3<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Scale {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<vek::Vec3<f32>> for Scale {
    fn as_ref(&self) -> &vek::Vec3<f32> {
        &self.0
    }
}

impl AsMut<vek::Vec3<f32>> for Scale {
    fn as_mut(&mut self) -> &mut vek::Vec3<f32> {
        &mut self.0
    }
}

impl From<Scale> for vek::Vec3<f32> {
    fn from(value: Scale) -> Self {
        value.0
    }
}

impl From<&Scale> for vek::Vec3<f32> {
    fn from(value: &Scale) -> Self {
        value.0
    }
}

impl From<vek::Vec3<f32>> for Scale {
    fn from(value: vek::Vec3<f32>) -> Self {
        Self(value)
    }
}

impl From<&vek::Vec3<f32>> for Scale {
    fn from(value: &vek::Vec3<f32>) -> Self {
        Self(*value)
    }
}

impl From<Scale> for vek::Mat4<f32> {
    fn from(value: Scale) -> Self {
        vek::Mat4::scaling_3d(value.0)
    }
}

impl From<Scale> for vek::Mat3<f32> {
    fn from(value: Scale) -> Self {
        vek::Mat3::scaling_3d(value.0)
    }
}

impl From<&Scale> for vek::Mat4<f32> {
    fn from(value: &Scale) -> Self {
        vek::Mat4::scaling_3d(value.0)
    }
}

impl From<&Scale> for vek::Mat3<f32> {
    fn from(value: &Scale) -> Self {
        vek::Mat3::scaling_3d(value.0)
    }
}