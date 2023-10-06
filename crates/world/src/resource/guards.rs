use std::{
    cell::{Ref, RefMut},
    ops::{Deref, DerefMut},
};

use crate::Resource;

// A read guard is an immutable reference to a resource
pub struct Read<'a, R: Resource>(pub(crate) Ref<'a, R>);

impl<R: Resource> Deref for Read<'_, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<R: Resource> AsRef<R> for Read<'_, R> {
    fn as_ref(&self) -> &R {
        &self.0
    }
}

impl<'a, R: Resource> Read<'a, R> {
    // Map a read guard to a mapped read shard
    pub fn map<T: 'static>(self, modify: impl FnOnce(&R) -> &T) -> ReadShard<'a, T> {
        ReadShard(Ref::map(self.0, modify))
    }
}

// A read shared is a sub-guard of a bigger read guard. Most of the time, it is used to read a mapped value
pub struct ReadShard<'a, T>(pub(crate) Ref<'a, T>);

impl<T> Deref for ReadShard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> AsRef<T> for ReadShard<'_, T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

// A write guard is a mutable reference to a resource
pub struct Write<'a, R: Resource>(pub(crate) RefMut<'a, R>);

impl<R: Resource> Deref for Write<'_, R> {
    type Target = R;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<R: Resource> DerefMut for Write<'_, R> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<R: Resource> AsMut<R> for Write<'_, R> {
    fn as_mut(&mut self) -> &mut R {
        &mut self.0
    }
}

impl<R: Resource> AsRef<R> for Write<'_, R> {
    fn as_ref(&self) -> &R {
        &self.0
    }
}

impl<'a, R: Resource> Write<'a, R> {
    // Map a write guard to a mapped write shard
    pub fn map<T: 'static>(self, modify: impl FnOnce(&mut R) -> &mut T) -> WriteShard<'a, T> {
        WriteShard(RefMut::map(self.0, modify))
    }
}

// A write shard is a sub-guard of a bigger write guard. Most of the time, it is used to write/read a mapped value
pub struct WriteShard<'a, T>(pub(crate) RefMut<'a, T>);

impl<T> Deref for WriteShard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for WriteShard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> AsMut<T> for WriteShard<'_, T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> AsRef<T> for WriteShard<'_, T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}
