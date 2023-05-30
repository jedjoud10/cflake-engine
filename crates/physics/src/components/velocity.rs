use ecs::Component;
use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut}, marker::PhantomData,
};
#[derive(Default, Clone, Copy, PartialEq, Component)]
#[repr(transparent)]
pub struct Velocity<Space: 'static>(vek::Vec3<f32>, PhantomData<Space>);

impl<Space: 'static> Velocity<Space> {
    // Construct a velocity with the given X unit velocity
    pub fn with_x(x: f32) -> Self {
        Self(vek::Vec3::new(x, 0.0, 0.0), PhantomData)
    }

    // Construct a velocity with the given Y unit velocity
    pub fn with_y(y: f32) -> Self {
        Self(vek::Vec3::new(0.0, y, 0.0), PhantomData)
    }

    // Construct a velocity with the given Z unit velocity
    pub fn with_z(z: f32) -> Self {
        Self(vek::Vec3::new(0.0, 0.0, z), PhantomData)
    }

    // Construct a velocity with the given X, Y, Z velocity
    pub fn with_xyz(x: f32, y: f32, z: f32) -> Self {
        Self((x, y, z).into(), PhantomData)
    }
}

impl<Space: 'static> Debug for Velocity<Space> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<Space: 'static> Display for Velocity<Space> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<Space: 'static> Deref for Velocity<Space> {
    type Target = vek::Vec3<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Space: 'static> DerefMut for Velocity<Space> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<Space: 'static> AsRef<vek::Vec3<f32>> for Velocity<Space> {
    fn as_ref(&self) -> &vek::Vec3<f32> {
        &self.0
    }
}

impl<Space: 'static> AsMut<vek::Vec3<f32>> for Velocity<Space> {
    fn as_mut(&mut self) -> &mut vek::Vec3<f32> {
        &mut self.0
    }
}

impl<Space: 'static> From<Velocity<Space>> for vek::Vec3<f32> {
    fn from(value: Velocity<Space>) -> Self {
        value.0
    }
}

impl<Space: 'static> From<&Velocity<Space>> for vek::Vec3<f32> {
    fn from(value: &Velocity<Space>) -> Self {
        value.0
    }
}

impl<Space: 'static> From<vek::Vec3<f32>> for Velocity<Space> {
    fn from(value: vek::Vec3<f32>) -> Self {
        Self(value, PhantomData)
    }
}

impl<Space: 'static> From<&vek::Vec3<f32>> for Velocity<Space> {
    fn from(value: &vek::Vec3<f32>) -> Self {
        Self(*value, PhantomData)
    }
}
