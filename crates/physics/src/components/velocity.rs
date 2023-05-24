use math::Scalar;

use ecs::Component;
use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut}, marker::PhantomData,
};

// Our target is the raw point (either 3D or 2D)
type Target = math::RawPoint;

#[derive(Default, Clone, Copy, PartialEq, Component)]
#[repr(transparent)]
pub struct Velocity<T: 'static>(Target, PhantomData<T>);

impl<T: 'static> Velocity<T> {
    // Construct a velocity with the given X unit velocity
    pub fn with_x(x: Scalar) -> Self {
        Self(vek::Vec3::new(x, 0.0, 0.0), PhantomData)
    }

    // Construct a velocity with the given Y unit velocity
    pub fn with_y(y: Scalar) -> Self {
        Self(vek::Vec3::new(0.0, y, 0.0), PhantomData)
    }

    // Construct a velocity with the given Z unit velocity
    pub fn with_z(z: Scalar) -> Self {
        Self(vek::Vec3::new(0.0, 0.0, z), PhantomData)
    }

    // Construct a velocity with the given X, Y, Z velocity
    pub fn with_xyz(x: Scalar, y: Scalar, z: Scalar) -> Self {
        Self((x, y, z).into(), PhantomData)
    }
}

impl<T: 'static> Debug for Velocity<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<T: 'static> Display for Velocity<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<T: 'static> Deref for Velocity<T> {
    type Target = Target;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: 'static> DerefMut for Velocity<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: 'static> AsRef<Target> for Velocity<T> {
    fn as_ref(&self) -> &Target {
        &self.0
    }
}

impl<T: 'static> AsMut<Target> for Velocity<T> {
    fn as_mut(&mut self) -> &mut Target {
        &mut self.0
    }
}

impl<T: 'static> From<Velocity<T>> for Target {
    fn from(value: Velocity<T>) -> Self {
        value.0
    }
}

impl<T: 'static> From<&Velocity<T>> for Target {
    fn from(value: &Velocity<T>) -> Self {
        value.0
    }
}

impl<T: 'static> From<Target> for Velocity<T> {
    fn from(value: Target) -> Self {
        Self(value, PhantomData)
    }
}

impl<T: 'static> From<&Target> for Velocity<T> {
    fn from(value: &Target) -> Self {
        Self(*value, PhantomData)
    }
}
