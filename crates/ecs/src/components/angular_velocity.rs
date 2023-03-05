use math::Scalar;

use crate::Component;
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

// Our target is the raw rotation (either 3D or 2D)
type Target = math::RawRotation;

#[derive(Default, Clone, Copy, PartialEq, Component)]
#[repr(transparent)]
pub struct AngularRotation(Target);

#[cfg(not(feature = "two-dim"))]
impl AngularRotation {
    // Construct an angular rotation using an X rotation (radians)
    pub fn angular_rotation_x(angle_radians: Scalar) -> Self {
        Self(vek::Quaternion::rotation_x(angle_radians))
    }

    // Construct an angular rotation using a Y rotation (radians)
    pub fn angular_rotation_y(angle_radians: Scalar) -> Self {
        Self(vek::Quaternion::rotation_y(angle_radians))
    }

    // Construct an angular rotation using a Z rotation (radians)
    pub fn angular_rotation_z(angle_radians: Scalar) -> Self {
        Self(vek::Quaternion::rotation_z(angle_radians))
    }
}

#[cfg(feature = "two-dim")]
impl AngularRotation {
    // Construct a 2D angular rotation using an angle (radians)
    pub fn angular_from_angle(angle_radians: Scalar) -> Self {
        Self(angle_radians)
    }
}

impl Debug for AngularRotation {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

#[cfg(feature = "two-dim")]
impl Display for AngularRotation {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Deref for AngularRotation {
    type Target = Target;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AngularRotation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<Target> for AngularRotation {
    fn as_ref(&self) -> &Target {
        &self.0
    }
}

impl AsMut<Target> for AngularRotation {
    fn as_mut(&mut self) -> &mut Target {
        &mut self.0
    }
}

impl From<AngularRotation> for Target {
    fn from(value: AngularRotation) -> Self {
        value.0
    }
}

impl From<&AngularRotation> for Target {
    fn from(value: &AngularRotation) -> Self {
        value.0
    }
}

impl From<Target> for AngularRotation {
    fn from(q: Target) -> Self {
        Self(q)
    }
}

impl From<&Target> for AngularRotation {
    fn from(q: &Target) -> Self {
        Self(*q)
    }
}

impl From<AngularRotation> for math::RawMatrix {
    fn from(value: AngularRotation) -> Self {
        value.0.into()
    }
}

impl From<&AngularRotation> for math::RawMatrix {
    fn from(value: &AngularRotation) -> Self {
        value.0.into()
    }
}
