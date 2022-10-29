use crate::Component;
use std::ops::{Add, Deref, DerefMut};

#[derive(Default, Clone, Copy, Component)]
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
        Self((x, y, z).into())
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

impl From<Location> for vek::Vec3<f32> {
    fn from(value: Location) -> Self {
        value.0
    }
}

impl From<&Location> for vek::Vec3<f32> {
    fn from(value: &Location) -> Self {
        value.0
    }
}

impl From<vek::Vec3<f32>> for Location {
    fn from(value: vek::Vec3<f32>) -> Self {
        Self(value)
    }
}

impl From<&vek::Vec3<f32>> for Location {
    fn from(value: &vek::Vec3<f32>) -> Self {
        Self(*value)
    }
}

impl From<Location> for vek::Mat4<f32> {
    fn from(value: Location) -> Self {
        vek::Mat4::translation_3d(value.0)
    }
}

impl From<&Location> for vek::Mat4<f32> {
    fn from(value: &Location) -> Self {
        vek::Mat4::translation_3d(value.0)
    }
}

impl Add<Location> for Location {
    type Output = Location;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
