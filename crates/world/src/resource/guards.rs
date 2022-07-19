use std::{cell::{RefMut, Ref}, ops::{Deref, DerefMut}};

use crate::Resource;

// A read guard is an immutable reference to a resource
pub struct Read<'a, R: Resource>(pub(crate) Ref<'a, R>);

impl<R: Resource> Deref for Read<'_, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<R: Resource> AsRef<R> for Read<'_, R> {
    fn as_ref(&self) -> &R {
        &*self.0
    }
}

// A write guard is a mutable reference to a resource
pub struct Write<'a, R: Resource>(pub(crate) RefMut<'a, R>);

impl<R: Resource> Deref for Write<'_, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<R: Resource> DerefMut for Write<'_, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

impl<R: Resource> AsMut<R> for Write<'_, R> {
    fn as_mut(&mut self) -> &mut R {
        &mut *self.0
    }
}

impl<R: Resource> AsRef<R> for Write<'_, R> {
    fn as_ref(&self) -> &R {
        &*self.0
    }
}