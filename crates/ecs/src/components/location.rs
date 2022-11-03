use crate::Component;
use std::ops::{Add, Deref, DerefMut};


// 2D location support
#[cfg(not(feature = "two-dim"))]
type Target = vek::Vec3<f32>;
#[cfg(feature = "two-dim")]
type Target = vek::Vec2<f32>;

// 2D matrix support
#[cfg(not(feature = "two-dim"))]
type Matrix = vek::Mat4<f32>;
#[cfg(feature = "two-dim")]
type Matrix = vek::Mat3<f32>;

#[derive(Default, Clone, Copy, Component)]
#[repr(transparent)]
pub struct Location(Target);

#[cfg(not(feature = "two-dim"))]
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

#[cfg(feature = "two-dim")]
impl Location {
    // Construct a scale at the given X unit position
    pub fn at_x(x: f32) -> Self {
        Self(vek::Vec2::new(x, 0.0))
    }

    // Construct a scale at the given Y unit position
    pub fn at_y(y: f32) -> Self {
        Self(vek::Vec2::new(0.0, y))
    }

    // Construct a scale at the given X, Y position
    pub fn at_xy(x: f32, y: f32) -> Self {
        Self((x, y).into())
    }
}

impl Deref for Location {
    type Target = Target;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Location {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<Target> for Location {
    fn as_ref(&self) -> &Target {
        &self.0
    }
}

impl AsMut<Target> for Location {
    fn as_mut(&mut self) -> &mut Target {
        &mut self.0
    }
}

impl From<Location> for Target {
    fn from(value: Location) -> Self {
        value.0
    }
}

impl From<&Location> for Target {
    fn from(value: &Location) -> Self {
        value.0
    }
}

impl From<Target> for Location {
    fn from(value: Target) -> Self {
        Self(value)
    }
}

impl From<&Target> for Location {
    fn from(value: &Target) -> Self {
        Self(*value)
    }
}

impl From<Location> for Matrix {
    fn from(value: Location) -> Self {
        #[cfg(not(feature = "two-dim"))]
        return vek::Mat4::translation_3d(value.0);

        #[cfg(feature = "two-dim")]
        return vek::Mat3::translation_2d(value.0);
    }
}

impl From<&Location> for vek::Mat4<f32> {
    fn from(value: &Location) -> Self {
        #[cfg(not(feature = "two-dim"))]
        return vek::Mat4::translation_3d(value.0);

        #[cfg(feature = "two-dim")]
        return vek::Mat3::translation_2d(value.0);
    }
}

impl Add<Location> for Location {
    type Output = Location;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
