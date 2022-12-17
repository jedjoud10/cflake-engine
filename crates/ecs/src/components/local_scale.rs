use math::Scalar;

use crate::Component;
use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};

// Our target is the raw point (either 3D or 2D)
type Target = math::RawPoint;

#[derive(Clone, Copy, Component)]
#[repr(transparent)]
pub struct LocalScale(Target);

impl Default for LocalScale {
    fn default() -> Self {
        Self(Target::one())
    }
}

#[cfg(not(feature = "two-dim"))]
impl LocalScale {
    // Construct a scale using an X width
    pub fn scale_x(width: Scalar) -> Self {
        Self(vek::Vec3::new(width, 1.0, 1.0))
    }

    // Construct a scale using a Y height
    pub fn scale_y(height: Scalar) -> Self {
        Self(vek::Vec3::new(1.0, height, 1.0))
    }

    // Construct a scale using a Z depth
    pub fn scale_z(depth: Scalar) -> Self {
        Self(vek::Vec3::new(1.0, 1.0, depth))
    }

    // Construct a scale with it's raw data
    pub fn scale_xyz(x: Scalar, y: Scalar, z: Scalar) -> Self {
        Self(vek::Vec3::new(x, y, z))
    }
}

#[cfg(feature = "two-dim")]
impl LocalScale {
    // Construct a scale using an X width
    pub fn scale_x(width: Scalar) -> Self {
        Self(vek::Vec2::new(width, 1.0))
    }

    // Construct a scale using a Y height
    pub fn scale_y(height: Scalar) -> Self {
        Self(vek::Vec2::new(1.0, height))
    }

    // Construct a scale with it's raw data
    pub fn scale_xy(x: Scalar, y: Scalar) -> Self {
        Self(vek::Vec2::new(x, y))
    }
}

impl Debug for LocalScale {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for LocalScale {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Deref for LocalScale {
    type Target = Target;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LocalScale {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<Target> for LocalScale {
    fn as_ref(&self) -> &Target {
        &self.0
    }
}

impl AsMut<Target> for LocalScale {
    fn as_mut(&mut self) -> &mut Target {
        &mut self.0
    }
}

impl From<LocalScale> for Target {
    fn from(value: LocalScale) -> Self {
        value.0
    }
}

impl From<&LocalScale> for Target {
    fn from(value: &LocalScale) -> Self {
        value.0
    }
}

impl From<Target> for LocalScale {
    fn from(value: Target) -> Self {
        Self(value)
    }
}

impl From<&Target> for LocalScale {
    fn from(value: &Target) -> Self {
        Self(*value)
    }
}

impl From<LocalScale> for math::RawMatrix {
    fn from(value: LocalScale) -> Self {
        vek::Mat4::scaling_3d(value.0)
    }
}

impl From<&LocalScale> for math::RawMatrix {
    fn from(value: &LocalScale) -> Self {
        #[cfg(not(feature = "two-dim"))]
        return vek::Mat4::scaling_3d(value.0);
        #[cfg(feature = "two-dim")]
        return vek::Mat3::scaling_2d(value.0);
    }
}
