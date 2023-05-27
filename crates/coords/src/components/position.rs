use ecs::Component;
use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

#[derive(Default, Clone, Copy, PartialEq, Component)]
#[repr(transparent)]
pub struct Position<T: 'static>(vek::Vec3<f32>, PhantomData<T>);

impl<T> Position<T> {
    // Construct a position at the given X unit position
    pub fn at_x(x: f32) -> Self {
        Self(vek::Vec3::new(x, 0.0, 0.0), PhantomData)
    }

    // Construct a position at the given Y unit position
    pub fn at_y(y: f32) -> Self {
        Self(vek::Vec3::new(0.0, y, 0.0), PhantomData)
    }

    // Construct a position at the given Z unit position
    pub fn at_z(z: f32) -> Self {
        Self(vek::Vec3::new(0.0, 0.0, z), PhantomData)
    }

    // Construct a position at the given X, Y, Z position
    pub fn at_xyz(x: f32, y: f32, z: f32) -> Self {
        Self((x, y, z).into(), PhantomData)
    }

    // Construct a position at the given X, Y, Z position (stored in an array)
    pub fn at_xyz_array(array: [f32; 3]) -> Self {
        Self::at_xyz(array[0], array[1], array[2])
    }
}

impl<T> Debug for Position<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<T> Display for Position<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<T> Deref for Position<T> {
    type Target = vek::Vec3<f32>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Position<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> AsRef<vek::Vec3<f32>> for Position<T> {
    fn as_ref(&self) -> &vek::Vec3<f32> {
        &self.0
    }
}

impl<T> AsMut<vek::Vec3<f32>> for Position<T> {
    fn as_mut(&mut self) -> &mut vek::Vec3<f32> {
        &mut self.0
    }
}

impl<T> From<Position<T>> for vek::Vec3<f32> {
    fn from(value: Position<T>) -> Self {
        value.0
    }
}

impl<T> From<&Position<T>> for vek::Vec3<f32> {
    fn from(value: &Position<T>) -> Self {
        value.0
    }
}

impl<T> From<vek::Vec3<f32>> for Position<T> {
    fn from(value: vek::Vec3<f32>) -> Self {
        Self(value, PhantomData)
    }
}

impl<T> From<&vek::Vec3<f32>> for Position<T> {
    fn from(value: &vek::Vec3<f32>) -> Self {
        Self(*value, PhantomData)
    }
}

impl<T> From<Position<T>> for vek::Mat4<f32> {
    fn from(value: Position<T>) -> Self {
        vek::Mat4::translation_3d(value.0)
    }
}

impl<T> From<&Position<T>> for vek::Mat4<f32> {
    fn from(value: &Position<T>) -> Self {
        vek::Mat4::translation_3d(value.0)
    }
}
