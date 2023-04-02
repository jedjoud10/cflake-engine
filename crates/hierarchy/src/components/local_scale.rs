use crate::Component;
use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};

// Our target is the scalar (uniform scale)
type Target = math::Scalar;

#[derive(Clone, Copy, PartialEq, Component)]
#[repr(transparent)]
pub struct LocalScale(Target);

impl Default for LocalScale {
    fn default() -> Self {
        Self::unit()
    }
}

impl LocalScale {
    // Construct a uniform scale with the given value
    pub fn uniform(scale: math::Scalar) -> Self {
        Self(scale)
    }

    // Construct a "unit" scale, aka default scale
    pub fn unit() -> Self {
        Self(1.0)
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
        #[cfg(not(feature = "two-dim"))]
        return vek::Mat4::scaling_3d(vek::Vec3::broadcast(value.0));
        #[cfg(feature = "two-dim")]
        return vek::Mat3::scaling_2d(vek::Vec2::broadcast(value.0));
    }
}

impl From<&LocalScale> for math::RawMatrix {
    fn from(value: &LocalScale) -> Self {
        #[cfg(not(feature = "two-dim"))]
        return vek::Mat4::scaling_3d(vek::Vec3::broadcast(value.0));
        #[cfg(feature = "two-dim")]
        return vek::Mat3::scaling_2d(vek::Vec2::broadcast(value.0));
    }
}
