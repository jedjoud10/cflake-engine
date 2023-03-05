use math::Scalar;

use crate::Component;
use std::{
    fmt::{Debug, Display},
    ops::{Add, Deref, DerefMut},
};

// Our target is the raw point (either 3D or 2D)
type Target = math::RawPoint;

#[derive(Default, Clone, Copy, PartialEq, Component)]
#[repr(transparent)]
pub struct Velocity(Target);

#[cfg(not(feature = "two-dim"))]
impl Velocity {
    // Construct a velocity with the given X unit velocity
    pub fn with_x(x: Scalar) -> Self {
        Self(vek::Vec3::new(x, 0.0, 0.0))
    }

    // Construct a velocity with the given Y unit velocity
    pub fn with_y(y: Scalar) -> Self {
        Self(vek::Vec3::new(0.0, y, 0.0))
    }

    // Construct a velocity with the given Z unit velocity
    pub fn with_z(z: Scalar) -> Self {
        Self(vek::Vec3::new(0.0, 0.0, z))
    }

    // Construct a velocity with the given X, Y, Z velocity
    pub fn with_xyz(x: Scalar, y: Scalar, z: Scalar) -> Self {
        Self((x, y, z).into())
    }
}

#[cfg(feature = "two-dim")]
impl Velocity {
    // Construct a velocity with the given X unit velocity
    pub fn with_x(x: Scalar) -> Self {
        Self(vek::Vec2::new(x, 0.0))
    }

    // Construct a velocity with the given Y unit velocity
    pub fn with_y(y: Scalar) -> Self {
        Self(vek::Vec2::new(0.0, y))
    }

    // Construct a velocity with the given X, Y velocity
    pub fn with_xy(x: Scalar, y: Scalar) -> Self {
        Self((x, y).into())
    }
}

impl Debug for Velocity {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for Velocity {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Deref for Velocity {
    type Target = Target;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Velocity {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<Target> for Velocity {
    fn as_ref(&self) -> &Target {
        &self.0
    }
}

impl AsMut<Target> for Velocity {
    fn as_mut(&mut self) -> &mut Target {
        &mut self.0
    }
}

impl From<Velocity> for Target {
    fn from(value: Velocity) -> Self {
        value.0
    }
}

impl From<&Velocity> for Target {
    fn from(value: &Velocity) -> Self {
        value.0
    }
}

impl From<Target> for Velocity {
    fn from(value: Target) -> Self {
        Self(value)
    }
}

impl From<&Target> for Velocity {
    fn from(value: &Target) -> Self {
        Self(*value)
    }
}
