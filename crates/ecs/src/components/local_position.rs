use crate::Component;
use std::{
    fmt::{Debug, Display},
    ops::{Add, Deref, DerefMut},
};

// Our target is the raw point (either 3D or 2D)
type Target = math::RawPoint;

#[derive(Default, Clone, Copy, Component)]
#[repr(transparent)]
pub struct LocalPosition(Target);

#[cfg(not(feature = "two-dim"))]
impl LocalPosition {
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
impl LocalPosition {
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

impl Debug for LocalPosition {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for LocalPosition {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Deref for LocalPosition {
    type Target = Target;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LocalPosition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<Target> for LocalPosition {
    fn as_ref(&self) -> &Target {
        &self.0
    }
}

impl AsMut<Target> for LocalPosition {
    fn as_mut(&mut self) -> &mut Target {
        &mut self.0
    }
}

impl From<LocalPosition> for Target {
    fn from(value: LocalPosition) -> Self {
        value.0
    }
}

impl From<&LocalPosition> for Target {
    fn from(value: &LocalPosition) -> Self {
        value.0
    }
}

impl From<Target> for LocalPosition {
    fn from(value: Target) -> Self {
        Self(value)
    }
}

impl From<&Target> for LocalPosition {
    fn from(value: &Target) -> Self {
        Self(*value)
    }
}

impl From<LocalPosition> for math::RawMatrix {
    fn from(value: LocalPosition) -> Self {
        #[cfg(not(feature = "two-dim"))]
        return vek::Mat4::translation_3d(value.0);

        #[cfg(feature = "two-dim")]
        return vek::Mat3::translation_2d(value.0);
    }
}

impl From<&LocalPosition> for vek::Mat4<f32> {
    fn from(value: &LocalPosition) -> Self {
        #[cfg(not(feature = "two-dim"))]
        return vek::Mat4::translation_3d(value.0);

        #[cfg(feature = "two-dim")]
        return vek::Mat3::translation_2d(value.0);
    }
}

impl Add<LocalPosition> for LocalPosition {
    type Output = LocalPosition;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}
