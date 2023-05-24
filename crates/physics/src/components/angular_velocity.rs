use math::Scalar;
use vek::Quaternion;

use ecs::Component;
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut}, marker::PhantomData,
};

// Our target is the raw rotation (either 3D or 2D)
type Target = math::RawRotation;

#[derive(Default, Clone, Copy, PartialEq, Component)]
#[repr(transparent)]
pub struct AngularVelocity<T: 'static>(Target, PhantomData<T>);

impl<T: 'static> AngularVelocity<T> {
    /// Construct an angular rotation using an X rotation (radians)
    pub fn angular_rotation_x(angle_radians: Scalar) -> Self {
        Self(vek::Quaternion::rotation_x(angle_radians), PhantomData)
    }

    /// Construct an angular rotation using a Y rotation (radians)
    pub fn angular_rotation_y(angle_radians: Scalar) -> Self {
        Self(vek::Quaternion::rotation_y(angle_radians), PhantomData)
    }

    /// Construct an angular rotation using a Z rotation (radians)
    pub fn angular_rotation_z(angle_radians: Scalar) -> Self {
        Self(vek::Quaternion::rotation_z(angle_radians), PhantomData)
    }
}

impl<T: 'static> Debug for AngularVelocity<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<T: 'static> Deref for AngularVelocity<T> {
    type Target = Target;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: 'static> DerefMut for AngularVelocity<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: 'static> AsRef<Target> for AngularVelocity<T> {
    fn as_ref(&self) -> &Target {
        &self.0
    }
}

impl<T: 'static> AsMut<Target> for AngularVelocity<T> {
    fn as_mut(&mut self) -> &mut Target {
        &mut self.0
    }
}

impl<T: 'static> From<AngularVelocity<T>> for Target {
    fn from(value: AngularVelocity<T>) -> Self {
        value.0
    }
}

impl<T: 'static> From<&AngularVelocity<T>> for Target {
    fn from(value: &AngularVelocity<T>) -> Self {
        value.0
    }
}

impl<T: 'static> From<Target> for AngularVelocity<T> {
    fn from(q: Target) -> Self {
        Self(q, PhantomData)
    }
}

impl<T: 'static> From<&Target> for AngularVelocity<T> {
    fn from(q: &Target) -> Self {
        Self(*q, PhantomData)
    }
}

impl<T: 'static> From<AngularVelocity<T>> for math::RawMatrix {
    fn from(value: AngularVelocity<T>) -> Self {
        value.0.into()
    }
}

impl<T: 'static> From<&AngularVelocity<T>> for math::RawMatrix {
    fn from(value: &AngularVelocity<T>) -> Self {
        value.0.into()
    }
}
