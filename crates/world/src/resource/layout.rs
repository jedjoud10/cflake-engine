use std::ops::{Deref, DerefMut};

use atomic_refcell::{AtomicRef, AtomicRefMut};

use crate::prelude::World;
use super::Resource;

/// An immutably/mutably borrowed resource
/// Could also represent a "partially" bound resource (Option<T>)
pub trait ResourceBorrow<'a>: Sized {
    /// Guarded borrowed (using RwLock guards)
    type Guarded<'b>: 'b;

    /// Access the inner value through the guard
    fn access(guard: &'a mut Self::Guarded<'_>) -> Self;
}

impl<'a, T: Resource> ResourceBorrow<'a> for &'a mut T {
    type Guarded<'b> = AtomicRefMut<'b, T>;

    fn access(guard: &'a mut Self::Guarded<'_>) -> Self {
        guard.deref_mut()
    }
}

impl<'a, T: Resource> ResourceBorrow<'a> for &'a T {
    type Guarded<'b> = AtomicRef<'b, T>;

    fn access(guard: &'a mut Self::Guarded<'_>) -> Self {
        (*guard).deref()
    }
}

impl<'a, T: Resource> ResourceBorrow<'a> for Option<&'a mut T> {
    type Guarded<'b> = Option<AtomicRefMut<'b, T>>;

    fn access(guard: &'a mut Self::Guarded<'_>) -> Self {
        guard.as_mut().map(|x| x.deref_mut())
    }
}

impl<'a, T: Resource> ResourceBorrow<'a> for Option<&'a T> {
    type Guarded<'b> = Option<AtomicRef<'b, T>>;

    fn access(guard: &'a mut Self::Guarded<'_>) -> Self {
        guard.as_ref().map(|x| x.deref())
    }
}

/// A combination of immutable and mutable resource borrows
/// This is the data that is given to systems for execution
pub trait ResourceLayout<'a> {
    /// Guarded tuple (using RwLock guards)
    type Guarded<'b>: 'b;

    /// Fetch the resource data from the world
    /// Returns None if there are missing resources
    fn fetch(world: &'a World) -> Self::Guarded<'a>;

    /// Convert the "Guarded" tuple to a borrowed tuple (which is "self")
    fn unguard(guard: &'a mut Self::Guarded<'_>) -> Self;
}

impl<'a, T: ResourceBorrow<'a>> ResourceLayout<'a> for T {
    type Guarded<'b> = T::Guarded<'b>;

    fn fetch(world: &'a World) -> Self::Guarded<'a> {
        todo!()
    }

    fn unguard(guard: &'a mut Self::Guarded<'_>) -> Self {
        T::access(guard)
    }
}