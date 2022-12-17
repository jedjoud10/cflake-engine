use crate::Component;
use std::{
    fmt::{Debug, Display},
    ops::{Add, Deref, DerefMut},
};

// Our target is the raw point (either 3D or 2D)
type Target = math::RawPoint;

#[derive(Default, Clone, Copy, Component)]
#[repr(transparent)]
pub struct Position(Target);

#[cfg(not(feature = "two-dim"))]
impl Position {
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
impl Position {
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

impl Debug for Position {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for Position {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Deref for Position {
    type Target = Target;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Position {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<Target> for Position {
    fn as_ref(&self) -> &Target {
        &self.0
    }
}

impl AsMut<Target> for Position {
    fn as_mut(&mut self) -> &mut Target {
        &mut self.0
    }
}

impl From<Position> for Target {
    fn from(value: Position) -> Self {
        value.0
    }
}

impl From<&Position> for Target {
    fn from(value: &Position) -> Self {
        value.0
    }
}

impl From<Target> for Position {
    fn from(value: Target) -> Self {
        Self(value)
    }
}

impl From<&Target> for Position {
    fn from(value: &Target) -> Self {
        Self(*value)
    }
}

impl From<Position> for math::RawMatrix {
    fn from(value: Position) -> Self {
        #[cfg(not(feature = "two-dim"))]
        return vek::Mat4::translation_3d(value.0);

        #[cfg(feature = "two-dim")]
        return vek::Mat3::translation_2d(value.0);
    }
}

impl From<&Position> for vek::Mat4<f32> {
    fn from(value: &Position) -> Self {
        #[cfg(not(feature = "two-dim"))]
        return vek::Mat4::translation_3d(value.0);

        #[cfg(feature = "two-dim")]
        return vek::Mat3::translation_2d(value.0);
    }
}

impl Add<Position> for Position {
    type Output = Position;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
