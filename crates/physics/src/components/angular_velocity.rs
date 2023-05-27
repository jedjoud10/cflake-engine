use vek::Quaternion;

use ecs::Component;
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut}, marker::PhantomData,
};

#[derive(Default, Clone, Copy, PartialEq, Component)]
#[repr(transparent)]
pub struct AngularVelocity<T: 'static>(vek::Vec3<f32>, PhantomData<T>);

impl<T: 'static> AngularVelocity<T> {
}

impl<T: 'static> Debug for AngularVelocity<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<T: 'static> Deref for AngularVelocity<T> {
    type Target = vek::Vec3<f32>;

    fn deref(&self) -> &vek::Vec3<f32> {
        &self.0
    }
}

impl<T: 'static> DerefMut for AngularVelocity<T> {
    fn deref_mut(&mut self) -> &mut vek::Vec3<f32> {
        &mut self.0
    }
}

impl<T: 'static> AsRef<vek::Vec3<f32>> for AngularVelocity<T> {
    fn as_ref(&self) -> &vek::Vec3<f32> {
        &self.0
    }
}

impl<T: 'static> AsMut<vek::Vec3<f32>> for AngularVelocity<T> {
    fn as_mut(&mut self) -> &mut vek::Vec3<f32> {
        &mut self.0
    }
}

impl<T: 'static> From<AngularVelocity<T>> for vek::Vec3<f32> {
    fn from(value: AngularVelocity<T>) -> Self {
        value.0
    }
}

impl<T: 'static> From<&AngularVelocity<T>> for vek::Vec3<f32> {
    fn from(value: &AngularVelocity<T>) -> Self {
        value.0
    }
}

impl<T: 'static> From<vek::Vec3<f32>> for AngularVelocity<T> {
    fn from(q: vek::Vec3<f32>) -> Self {
        Self(q, PhantomData)
    }
}

impl<T: 'static> From<&vek::Vec3<f32>> for AngularVelocity<T> {
    fn from(q: &vek::Vec3<f32>) -> Self {
        Self(*q, PhantomData)
    }
}