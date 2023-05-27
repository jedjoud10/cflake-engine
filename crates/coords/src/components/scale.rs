use ecs::Component;
use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

#[derive(Clone, Copy, PartialEq, Component)]
#[repr(transparent)]
pub struct Scale<T: 'static>(f32, PhantomData<T>);

impl<T> Default for Scale<T> {
    fn default() -> Self {
        Self::unit()
    }
}

impl<T> Scale<T> {
    // Construct a uniform scale with the given value
    pub fn uniform(scale: f32) -> Self {
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
    type Target = f32;

    fn deref(&self) -> &f32 {
        &self.0
    }
}

impl<T> DerefMut for Scale<T> {
    fn deref_mut(&mut self) -> &mut f32 {
        &mut self.0
    }
}

impl<T> AsRef<f32> for Scale<T> {
    fn as_ref(&self) -> &f32 {
        &self.0
    }
}

impl<T> AsMut<f32> for Scale<T> {
    fn as_mut(&mut self) -> &mut f32 {
        &mut self.0
    }
}

impl<T> From<Scale<T>> for f32 {
    fn from(value: Scale<T>) -> Self {
        value.0
    }
}

impl<T> From<&Scale<T>> for f32 {
    fn from(value: &Scale<T>) -> Self {
        value.0
    }
}

impl<T> From<f32> for Scale<T> {
    fn from(value: f32) -> Self {
        Self(value, PhantomData)
    }
}

impl<T> From<&f32> for Scale<T> {
    fn from(value: &f32) -> Self {
        Self(*value, PhantomData)
    }
}

impl<T> From<Scale<T>> for vek::Mat4<f32> {
    fn from(value: Scale<T>) -> Self {
        vek::Mat4::scaling_3d(vek::Vec3::broadcast(value.0))
    }
}

impl<T> From<&Scale<T>> for vek::Mat4<f32> {
    fn from(value: &Scale<T>) -> Self {
        vek::Mat4::scaling_3d(vek::Vec3::broadcast(value.0))
    }
}
