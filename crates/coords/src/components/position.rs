use math::Scalar;

use ecs::Component;
use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};

// Our target is the raw point (either 3D or 2D)
type Target = math::RawPoint;

#[derive(Default, Clone, Copy, PartialEq, Component)]
#[repr(transparent)]
pub struct Position(Target);

#[cfg(not(feature = "two-dim"))]
impl Position {
    // Construct a position at the given X unit position
    pub fn at_x(x: Scalar) -> Self {
        Self(vek::Vec3::new(x, 0.0, 0.0))
    }

    // Construct a position at the given Y unit position
    pub fn at_y(y: Scalar) -> Self {
        Self(vek::Vec3::new(0.0, y, 0.0))
    }

    // Construct a position at the given Z unit position
    pub fn at_z(z: Scalar) -> Self {
        Self(vek::Vec3::new(0.0, 0.0, z))
    }

    // Construct a position at the given X, Y, Z position
    pub fn at_xyz(x: Scalar, y: Scalar, z: Scalar) -> Self {
        Self((x, y, z).into())
    }

    // Construct a position at the given X, Y, Z position (stored in an array)
    pub fn at_xyz_array(array: [Scalar; 3]) -> Self {
        Self::at_xyz(array[0], array[1], array[2])
    }
}

#[cfg(feature = "two-dim")]
impl Position {
    // Construct a position at the given X unit position
    pub fn at_x(x: Scalar) -> Self {
        Self(vek::Vec2::new(x, 0.0))
    }

    // Construct a position at the given Y unit position
    pub fn at_y(y: Scalar) -> Self {
        Self(vek::Vec2::new(0.0, y))
    }

    // Construct a position at the given X, Y position
    pub fn at_xy(x: Scalar, y: Scalar) -> Self {
        Self((x, y).into())
    }
}

impl Debug for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl From<&Position> for vek::Mat4<Scalar> {
    fn from(value: &Position) -> Self {
        #[cfg(not(feature = "two-dim"))]
        return vek::Mat4::translation_3d(value.0);

        #[cfg(feature = "two-dim")]
        return vek::Mat3::translation_2d(value.0);
    }
}
