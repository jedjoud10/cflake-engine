use std::ops::{Deref, DerefMut};
use crate::IntoMatrix;

#[derive(Default, Clone, Copy)]
pub struct Location(vek::Vec3<f32>);

impl Location {
    // Construct a scale at the given X unit position
    pub fn at_x(x: f32) -> Self {
        Self(vek::Vec3::new(x, 0.0, 0.0))
    }

    // Construct a scale at the given Y unit position
    pub fn at_y(y: f32) -> Self {
        Self(vek::Vec3::new(0.0, y, 0.0))
    }

    // Construct a scale at the given Z unit position
    pub fn at_z(z: f32) -> Self {
        Self(vek::Vec3::new(0.0, 0.0, z))
    }

    // Construct a scale at the given X, Y, Z position
    pub fn at_xyz(x: f32, y: f32, z: f32) -> Self {
        Self::from((x, y, z))
    }
}

impl IntoMatrix for Location {
    fn matrix(self) -> vek::Mat4<f32> {
        vek::Mat4::<f32>::translation_3d(self.0)
    }
}

impl Deref for Location {
    type Target = vek::Vec3<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Location {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<vek::Vec3<f32>> for Location {
    fn as_ref(&self) -> &vek::Vec3<f32> {
        &self.0
    }
}

impl AsMut<vek::Vec3<f32>> for Location {
    fn as_mut(&mut self) -> &mut vek::Vec3<f32> {
        &mut self.0
    }
}

impl Into<vek::Vec3<f32>> for Location {
    fn into(self) -> vek::Vec3<f32> {
        self.0
    }
}

impl Into<(f32, f32, f32)> for Location {
    fn into(self) -> (f32, f32, f32) {
        self.0.into_tuple()
    }
}

impl From<vek::Vec3<f32>> for Location {
    fn from(l: vek::Vec3<f32>) -> Self {
        Self(l)
    }
}

impl From<(f32, f32, f32)> for Location {
    fn from(l: (f32, f32, f32)) -> Self {
        Self(l.into())
    }
}