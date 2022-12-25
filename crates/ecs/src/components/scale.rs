use math::Scalar;

use crate::Component;
use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};

// Our target is the scalar (uniform scale)
type Target = math::Scalar;

#[derive(Clone, Copy, PartialEq, Component)]
#[repr(transparent)]
pub struct Scale(Target);

impl Default for Scale {
    fn default() -> Self {
        Self::unit()
    }
}

impl Scale {
    // Construct a uniform scale with the given value
    pub fn uniform(scale: math::Scalar) -> Self {
        Self(scale)
    }

    // Construct a "unit" scale, aka default scale
    pub fn unit() -> Self {
        Self(1.0)
    }
}

impl Debug for Scale {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for Scale {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Deref for Scale {
    type Target = Target;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Scale {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<Target> for Scale {
    fn as_ref(&self) -> &Target {
        &self.0
    }
}

impl AsMut<Target> for Scale {
    fn as_mut(&mut self) -> &mut Target {
        &mut self.0
    }
}

impl From<Scale> for Target {
    fn from(value: Scale) -> Self {
        value.0
    }
}

impl From<&Scale> for Target {
    fn from(value: &Scale) -> Self {
        value.0
    }
}

impl From<Target> for Scale {
    fn from(value: Target) -> Self {
        Self(value)
    }
}

impl From<&Target> for Scale {
    fn from(value: &Target) -> Self {
        Self(*value)
    }
}

impl From<Scale> for math::RawMatrix {
    fn from(value: Scale) -> Self {
        #[cfg(not(feature = "two-dim"))]
        return vek::Mat4::scaling_3d(vek::Vec3::broadcast(value.0));
        #[cfg(feature = "two-dim")]
        return vek::Mat3::scaling_2d(vek::Vec2::broadcast(value.0));
    }
}

impl From<&Scale> for math::RawMatrix {
    fn from(value: &Scale) -> Self {
        #[cfg(not(feature = "two-dim"))]
        return vek::Mat4::scaling_3d(vek::Vec3::broadcast(value.0));
        #[cfg(feature = "two-dim")]
        return vek::Mat3::scaling_2d(vek::Vec2::broadcast(value.0));
    }
}
