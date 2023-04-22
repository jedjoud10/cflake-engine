use math::Scalar;

use ecs::Component;
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

// Our target is the raw rotation (either 3D or 2D)
type Target = math::RawRotation;

#[derive(Default, Clone, Copy, PartialEq, Component)]
#[repr(transparent)]
pub struct Rotation(Target);

#[cfg(not(feature = "two-dim"))]
impl Rotation {
    // Create a new rotation based on the RAW quaternion components (stored in an array)
    pub fn new_xyzw_array(array: [Scalar; 4]) -> Self {
        Self::new_xyzw(array[0], array[1], array[2], array[3])
    }

    // Creates a new rotation based on the RAW quaternion components
    // Only use this if you know what you are doing
    pub fn new_xyzw(x: Scalar, y: Scalar, z: Scalar, w: Scalar) -> Self {
        Self(Target::from_xyzw(x, y, z, w))
    }

    // Calculate the forward vector (-Z)
    pub fn forward(&self) -> vek::Vec3<Scalar> {
        vek::Mat4::from(self).mul_point(-vek::Vec3::unit_z())
    }

    // Calculate the up vector (+Y)
    pub fn up(&self) -> vek::Vec3<Scalar> {
        vek::Mat4::from(self).mul_point(vek::Vec3::unit_y())
    }

    // Calculate the right vector (+X)
    pub fn right(&self) -> vek::Vec3<Scalar> {
        vek::Mat4::from(self).mul_point(vek::Vec3::unit_x())
    }

    // Construct a rotation using an X rotation (radians)
    pub fn rotation_x(angle_radians: Scalar) -> Self {
        Self(vek::Quaternion::rotation_x(angle_radians))
    }

    // Construct a rotation using a Y rotation (radians)
    pub fn rotation_y(angle_radians: Scalar) -> Self {
        Self(vek::Quaternion::rotation_y(angle_radians))
    }

    // Construct a rotation using a Z rotation (radians)
    pub fn rotation_z(angle_radians: Scalar) -> Self {
        Self(vek::Quaternion::rotation_z(angle_radians))
    }

    // Construct a rotation that is looking directly down (forward => (0, -1, 0))
    pub fn looking_down() -> Self {
        let scalar: Scalar = 90.0;
        Self::rotation_x(scalar.to_radians())
    }

    // Construct a rotation that is looking directly up (forward => (0, 1, 0))
    pub fn looking_up() -> Self {
        let scalar: Scalar = -90.0;
        Self::rotation_x(scalar.to_radians())
    }

    // Construct a rotation that is looking directly right (forward => (1, 0, 0))
    // TODO: Not sure if this is it or if I should inver it
    pub fn looking_right() -> Self {
        let scalar: Scalar = 90.0;
        Self::rotation_y(scalar.to_radians())
    }
}

#[cfg(feature = "two-dim")]
impl Rotation {
    // Calculate the forward vector (-Z)
    pub fn forward(&self) -> vek::Vec2<Scalar> {
        vek::Mat3::from(self).mul_point(-vek::Vec2::unit_x())
    }

    // Calculate the up vector (+Y)
    pub fn up(&self) -> vek::Vec2<Scalar> {
        vek::Mat3::from(self).mul_point(vek::Vec2::unit_y())
    }

    // Construct a 2D rotation using an angle (radians)
    pub fn from_angle(angle_radians: Scalar) -> Self {
        Self(angle_radians)
    }

    // Construct a rotation that is looking directly down (forward => (0, -1))
    pub fn looking_down() -> Self {
        let scalar: Scalar = 90.0;
        Self::from_angle(scalar)
    }

    // Construct a rotation that is looking directly up (forward => (0, 1))
    pub fn looking_up() -> Self {
        let scalar: Scalar = -90.0;
        Self::from_angle(scalar)
    }

    // Construct a rotation that is looking directly left (forward => (-1, 0))
    // TODO: Test
    pub fn looking_left() -> Self {
        Self::from_angle(-180.to_radians())
    }

    // Construct a rotation that is looking directly right (forward => (1, 0))
    // TODO: Test
    pub fn looking_right() -> Self {
        let scalar: Scalar = 0.0;
        Self::from_angle(scalar)
    }

    // Mix two rotation together using a lerp value
    pub fn mix(self, other: Self, t: Scalar) -> Self {
        let a = self.0;
        let b = self.1;
        a * t + (b * (1.0 - t))
    }
}

impl Debug for Rotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

#[cfg(feature = "two-dim")]
impl Display for Rotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Deref for Rotation {
    type Target = Target;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Rotation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<Target> for Rotation {
    fn as_ref(&self) -> &Target {
        &self.0
    }
}

impl AsMut<Target> for Rotation {
    fn as_mut(&mut self) -> &mut Target {
        &mut self.0
    }
}

impl From<Rotation> for Target {
    fn from(value: Rotation) -> Self {
        value.0
    }
}

impl From<&Rotation> for Target {
    fn from(value: &Rotation) -> Self {
        value.0
    }
}

impl From<Target> for Rotation {
    fn from(q: Target) -> Self {
        Self(q)
    }
}

impl From<&Target> for Rotation {
    fn from(q: &Target) -> Self {
        Self(*q)
    }
}

impl From<Rotation> for math::RawMatrix {
    fn from(value: Rotation) -> Self {
        value.0.into()
    }
}

impl From<&Rotation> for math::RawMatrix {
    fn from(value: &Rotation) -> Self {
        value.0.into()
    }
}
