use ecs::Component;
use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut}, marker::PhantomData,
};
#[derive(Default, Clone, Copy, PartialEq, Component)]
#[repr(transparent)]
pub struct Velocity<T: 'static>(vek::Vec3<f32>, PhantomData<T>);

impl<T: 'static> Velocity<T> {
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
    type Target = vek::Vec3<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: 'static> DerefMut for Velocity<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: 'static> AsRef<vek::Vec3<f32>> for Velocity<T> {
    fn as_ref(&self) -> &vek::Vec3<f32> {
        &self.0
    }
}

impl<T: 'static> AsMut<vek::Vec3<f32>> for Velocity<T> {
    fn as_mut(&mut self) -> &mut vek::Vec3<f32> {
        &mut self.0
    }
}

impl<T: 'static> From<Velocity<T>> for vek::Vec3<f32> {
    fn from(value: Velocity<T>) -> Self {
        value.0
    }
}

impl<T: 'static> From<&Velocity<T>> for vek::Vec3<f32> {
    fn from(value: &Velocity<T>) -> Self {
        value.0
    }
}

impl<T: 'static> From<vek::Vec3<f32>> for Velocity<T> {
    fn from(value: vek::Vec3<f32>) -> Self {
        Self(value, PhantomData)
    }
}

impl<T: 'static> From<&vek::Vec3<f32>> for Velocity<T> {
    fn from(value: &vek::Vec3<f32>) -> Self {
        Self(*value, PhantomData)
    }
}
