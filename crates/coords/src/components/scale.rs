use ecs::Component;
use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

/// Uniform scale component that allows entities to have a scale.
#[derive(Clone, Copy, PartialEq, Component)]
#[repr(transparent)]
pub struct Scale<Space: 'static>(f32, PhantomData<Space>);

impl<Space> Default for Scale<Space> {
    fn default() -> Self {
        Self::unit()
    }
}

impl<Space> Scale<Space> {
    /// Construct a uniform scale with the given value.
    pub fn uniform(scale: f32) -> Self {
        Self(scale, PhantomData)
    }

    /// Construct a "unit" scale, aka default scale.
    pub fn unit() -> Self {
        Self(1.0, PhantomData)
    }
}

impl<Space> Debug for Scale<Space> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<Space> Display for Scale<Space> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<Space> Deref for Scale<Space> {
    type Target = f32;

    fn deref(&self) -> &f32 {
        &self.0
    }
}

impl<Space> DerefMut for Scale<Space> {
    fn deref_mut(&mut self) -> &mut f32 {
        &mut self.0
    }
}

impl<Space> AsRef<f32> for Scale<Space> {
    fn as_ref(&self) -> &f32 {
        &self.0
    }
}

impl<Space> AsMut<f32> for Scale<Space> {
    fn as_mut(&mut self) -> &mut f32 {
        &mut self.0
    }
}

impl<Space> From<Scale<Space>> for f32 {
    fn from(value: Scale<Space>) -> Self {
        value.0
    }
}

impl<Space> From<&Scale<Space>> for f32 {
    fn from(value: &Scale<Space>) -> Self {
        value.0
    }
}

impl<Space> From<f32> for Scale<Space> {
    fn from(value: f32) -> Self {
        Self(value, PhantomData)
    }
}

impl<Space> From<&f32> for Scale<Space> {
    fn from(value: &f32) -> Self {
        Self(*value, PhantomData)
    }
}

impl<Space> From<Scale<Space>> for vek::Mat4<f32> {
    fn from(value: Scale<Space>) -> Self {
        vek::Mat4::scaling_3d(vek::Vec3::broadcast(value.0))
    }
}

impl<Space> From<&Scale<Space>> for vek::Mat4<f32> {
    fn from(value: &Scale<Space>) -> Self {
        vek::Mat4::scaling_3d(vek::Vec3::broadcast(value.0))
    }
}
