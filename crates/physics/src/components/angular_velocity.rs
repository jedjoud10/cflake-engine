use math::Scalar;
use vek::Quaternion;

use ecs::Component;
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

// Our target is the raw rotation (either 3D or 2D)
type Target = math::RawRotation;

#[derive(Default, Clone, Copy, PartialEq, Component)]
#[repr(transparent)]
pub struct AngularVelocity(Target);

#[cfg(not(feature = "two-dim"))]
impl AngularVelocity {
    /// Construct an angular rotation using an X rotation (radians)
    pub fn angular_rotation_x(angle_radians: Scalar) -> Self {
        Self(vek::Quaternion::rotation_x(angle_radians))
    }

    /// Construct an angular rotation using a Y rotation (radians)
    pub fn angular_rotation_y(angle_radians: Scalar) -> Self {
        Self(vek::Quaternion::rotation_y(angle_radians))
    }

    /// Construct an angular rotation using a Z rotation (radians)
    pub fn angular_rotation_z(angle_radians: Scalar) -> Self {
        Self(vek::Quaternion::rotation_z(angle_radians))
    }

    /// Construct an angular rotation using a rotation (radians)
    /// YOU HAVE FALLEN FOR THE TRAP LOGAN
    /// Fyi, xyz within the quaternion represents a complex number so you can't just use angles for it
    /// I have fixed it for u bby (srry)
    pub fn angular_rotation_xyz(
        x_angle_radians: Scalar,
        y_angle_radians: Scalar,
        z_angle_radians: Scalar,
    ) -> Self {
        let mut q: Quaternion<f32> = Quaternion::identity();
        q.rotate_x(x_angle_radians);
        q.rotate_y(y_angle_radians);
        q.rotate_z(z_angle_radians);

        /*
        let q: Quaternion<f32> = Quaternion { x: x_angle_radians, y: y_angle_radians, z: z_angle_radians, w: () };
        */

        Self(q)
    }
}

#[cfg(feature = "two-dim")]
impl AngularVelocity {
    // Construct a 2D angular rotation using an angle (radians)
    pub fn angular_from_angle(angle_radians: Scalar) -> Self {
        Self(angle_radians)
    }
}

impl Debug for AngularVelocity {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

#[cfg(feature = "two-dim")]
impl Display for AngularVelocity {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Deref for AngularVelocity {
    type Target = Target;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AngularVelocity {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<Target> for AngularVelocity {
    fn as_ref(&self) -> &Target {
        &self.0
    }
}

impl AsMut<Target> for AngularVelocity {
    fn as_mut(&mut self) -> &mut Target {
        &mut self.0
    }
}

impl From<AngularVelocity> for Target {
    fn from(value: AngularVelocity) -> Self {
        value.0
    }
}

impl From<&AngularVelocity> for Target {
    fn from(value: &AngularVelocity) -> Self {
        value.0
    }
}

impl From<Target> for AngularVelocity {
    fn from(q: Target) -> Self {
        Self(q)
    }
}

impl From<&Target> for AngularVelocity {
    fn from(q: &Target) -> Self {
        Self(*q)
    }
}

impl From<AngularVelocity> for math::RawMatrix {
    fn from(value: AngularVelocity) -> Self {
        value.0.into()
    }
}

impl From<&AngularVelocity> for math::RawMatrix {
    fn from(value: &AngularVelocity) -> Self {
        value.0.into()
    }
}
