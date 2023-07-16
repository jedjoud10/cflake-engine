use vek::Quaternion;

use ecs::Component;
use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

#[derive(Default, Clone, Copy, PartialEq, Component)]
#[repr(transparent)]
pub struct AngularVelocity<Space: 'static>(vek::Vec3<f32>, PhantomData<Space>);

impl<Space: 'static> AngularVelocity<Space> {}

impl<Space: 'static> Debug for AngularVelocity<Space> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<Space: 'static> Deref for AngularVelocity<Space> {
    type Target = vek::Vec3<f32>;

    fn deref(&self) -> &vek::Vec3<f32> {
        &self.0
    }
}

impl<Space: 'static> DerefMut for AngularVelocity<Space> {
    fn deref_mut(&mut self) -> &mut vek::Vec3<f32> {
        &mut self.0
    }
}

impl<Space: 'static> AsRef<vek::Vec3<f32>> for AngularVelocity<Space> {
    fn as_ref(&self) -> &vek::Vec3<f32> {
        &self.0
    }
}

impl<Space: 'static> AsMut<vek::Vec3<f32>> for AngularVelocity<Space> {
    fn as_mut(&mut self) -> &mut vek::Vec3<f32> {
        &mut self.0
    }
}

impl<Space: 'static> From<AngularVelocity<Space>> for vek::Vec3<f32> {
    fn from(value: AngularVelocity<Space>) -> Self {
        value.0
    }
}

impl<Space: 'static> From<&AngularVelocity<Space>> for vek::Vec3<f32> {
    fn from(value: &AngularVelocity<Space>) -> Self {
        value.0
    }
}

impl<Space: 'static> From<vek::Vec3<f32>> for AngularVelocity<Space> {
    fn from(q: vek::Vec3<f32>) -> Self {
        Self(q, PhantomData)
    }
}

impl<Space: 'static> From<&vek::Vec3<f32>> for AngularVelocity<Space> {
    fn from(q: &vek::Vec3<f32>) -> Self {
        Self(*q, PhantomData)
    }
}
