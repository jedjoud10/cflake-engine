use std::ops::{Deref, DerefMut};
use crate::IntoMatrix;

#[derive(Clone, Copy)]
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

impl IntoMatrix for Scale {
    fn matrix(self) -> vek::Mat4<f32> {
        vek::Mat4::<f32>::scaling_3d(self.0)
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

impl Into<vek::Vec3<f32>> for Scale {
    fn into(self) -> vek::Vec3<f32> {
        self.0
    }
}

impl Into<(f32, f32, f32)> for Scale {
    fn into(self) -> (f32, f32, f32) {
        self.0.into_tuple()
    }
}

impl From<vek::Vec3<f32>> for Scale {
    fn from(l: vek::Vec3<f32>) -> Self {
        Self(l)
    }
}

impl From<(f32, f32, f32)> for Scale {
    fn from(l: (f32, f32, f32)) -> Self {
        Self(l.into())
    }
}