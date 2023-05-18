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
pub struct Position<T: 'static>(Target, PhantomData<T>);

impl<T> Position<T> {
    // Construct a position at the given X unit position
    pub fn at_x(x: Scalar) -> Self {
        Self(vek::Vec3::new(x, 0.0, 0.0), PhantomData)
    }

    // Construct a position at the given Y unit position
    pub fn at_y(y: Scalar) -> Self {
        Self(vek::Vec3::new(0.0, y, 0.0), PhantomData)
    }

    // Construct a position at the given Z unit position
    pub fn at_z(z: Scalar) -> Self {
        Self(vek::Vec3::new(0.0, 0.0, z), PhantomData)
    }

    // Construct a position at the given X, Y, Z position
    pub fn at_xyz(x: Scalar, y: Scalar, z: Scalar) -> Self {
        Self((x, y, z).into(), PhantomData)
    }

    // Construct a position at the given X, Y, Z position (stored in an array)
    pub fn at_xyz_array(array: [Scalar; 3]) -> Self {
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
    type Target = Target;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Position<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> AsRef<Target> for Position<T> {
    fn as_ref(&self) -> &Target {
        &self.0
    }
}

impl<T> AsMut<Target> for Position<T> {
    fn as_mut(&mut self) -> &mut Target {
        &mut self.0
    }
}

impl<T> From<Position<T>> for Target {
    fn from(value: Position<T>) -> Self {
        value.0
    }
}

impl<T> From<&Position<T>> for Target {
    fn from(value: &Position<T>) -> Self {
        value.0
    }
}

impl<T> From<Target> for Position<T> {
    fn from(value: Target) -> Self {
        Self(value, PhantomData)
    }
}

impl<T> From<&Target> for Position<T> {
    fn from(value: &Target) -> Self {
        Self(*value, PhantomData)
    }
}

impl<T> From<Position<T>> for math::RawMatrix {
    fn from(value: Position<T>) -> Self {
        return vek::Mat4::translation_3d(value.0);
    }
}

impl<T> From<&Position<T>> for vek::Mat4<Scalar> {
    fn from(value: &Position<T>) -> Self {
        return vek::Mat4::translation_3d(value.0);
    }
}
