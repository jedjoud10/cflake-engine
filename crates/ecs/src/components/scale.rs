use crate::Component;
use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};

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

#[derive(Clone, Copy, Component)]
#[repr(transparent)]
pub struct Scale(Target);

impl Default for Scale {
    fn default() -> Self {
        Self(Target::one())
    }
}

#[cfg(not(feature = "two-dim"))]
impl Scale {
    // Construct a scale using an X width
    pub fn scale_x(width: f32) -> Self {
        Self(vek::Vec3::new(width, 1.0, 1.0))
    }

    // Construct a scale using a Y height
    pub fn scale_y(height: f32) -> Self {
        Self(vek::Vec3::new(1.0, height, 1.0))
    }

    // Construct a scale using a Z depth
    pub fn scale_z(depth: f32) -> Self {
        Self(vek::Vec3::new(1.0, 1.0, depth))
    }

    // Construct a scale with it's raw data
    pub fn scale_xyz(x: f32, y: f32, z: f32) -> Self {
        Self(vek::Vec3::new(x, y, z))
    }
}

#[cfg(feature = "two-dim")]
impl Scale {
    // Construct a scale using an X width
    pub fn scale_x(width: f32) -> Self {
        Self(vek::Vec2::new(width, 1.0))
    }

    // Construct a scale using a Y height
    pub fn scale_y(height: f32) -> Self {
        Self(vek::Vec2::new(1.0, height))
    }

    // Construct a scale with it's raw data
    pub fn scale_xy(x: f32, y: f32) -> Self {
        Self(vek::Vec2::new(x, y))
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

impl From<Scale> for Matrix {
    fn from(value: Scale) -> Self {
        vek::Mat4::scaling_3d(value.0)
    }
}

impl From<&Scale> for Matrix {
    fn from(value: &Scale) -> Self {
        #[cfg(not(feature = "two-dim"))]
        return vek::Mat4::scaling_3d(value.0);
        #[cfg(feature = "two-dim")]
        return vek::Mat3::scaling_2d(value.0);
    }
}
