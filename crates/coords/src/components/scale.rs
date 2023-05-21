use ecs::Component;
use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

// Our target is the scalar (uniform scale)
type Target = math::Scalar;

#[derive(Clone, Copy, PartialEq, Component)]
#[repr(transparent)]
pub struct Scale<T: 'static>(Target, PhantomData<T>);

impl<T> Default for Scale<T> {
    fn default() -> Self {
        Self::unit()
    }
}

impl<T> Scale<T> {
    // Construct a uniform scale with the given value
    pub fn uniform(scale: math::Scalar) -> Self {
        Self(scale, PhantomData)
    }

    // Construct a "unit" scale, aka default scale
    pub fn unit() -> Self {
        Self(1.0, PhantomData)
    }
}

impl<T> Debug for Scale<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<T> Display for Scale<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<T> Deref for Scale<T> {
    type Target = Target;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Scale<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> AsRef<Target> for Scale<T> {
    fn as_ref(&self) -> &Target {
        &self.0
    }
}

impl<T> AsMut<Target> for Scale<T> {
    fn as_mut(&mut self) -> &mut Target {
        &mut self.0
    }
}

impl<T> From<Scale<T>> for Target {
    fn from(value: Scale<T>) -> Self {
        value.0
    }
}

impl<T> From<&Scale<T>> for Target {
    fn from(value: &Scale<T>) -> Self {
        value.0
    }
}

impl<T> From<Target> for Scale<T> {
    fn from(value: Target) -> Self {
        Self(value, PhantomData)
    }
}

impl<T> From<&Target> for Scale<T> {
    fn from(value: &Target) -> Self {
        Self(*value, PhantomData)
    }
}

impl<T> From<Scale<T>> for math::RawMatrix {
    fn from(value: Scale<T>) -> Self {
        return vek::Mat4::scaling_3d(vek::Vec3::broadcast(value.0));
    }
}

impl<T> From<&Scale<T>> for math::RawMatrix {
    fn from(value: &Scale<T>) -> Self {
        return vek::Mat4::scaling_3d(vek::Vec3::broadcast(value.0));
    }
}
