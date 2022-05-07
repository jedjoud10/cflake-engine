use crate::{registry, Archetype, Component, Entity, Mask};
use std::{
    ops::{BitAnd, BitOr},
    ptr::NonNull,
};

// Layout access that contain the normal mask and writing mask
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LayoutAccess(Mask, Mask);

impl LayoutAccess {
    // No layout access at all
    pub const fn none() -> Self {
        Self(Mask::zero(), Mask::zero())
    }

    // Get the normal mask
    pub fn reading(&self) -> Mask {
        self.0
    }
    // Get the writing mask
    pub fn writing(&self) -> Mask {
        self.1
    }
}

impl BitOr for LayoutAccess {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0, self.1 | rhs.1)
    }
}

impl BitAnd for LayoutAccess {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0, self.1 & rhs.1)
    }
}

// Trait that will be implmenented for &T and &mut T where T is a component or entity
pub trait PtrReader<'a> {
    type Item: 'static;

    // Offset an base ptr by a bundle offset and read it
    fn offset(ptr: NonNull<Self::Item>, bundle: usize) -> Self;

    // Get the normal component mask AND writing mask, combined into a single layout mask, if possible
    fn access() -> LayoutAccess;

    // Get the corresponding base pointer from an archetype
    fn fetch(archetype: &Archetype) -> NonNull<Self::Item>;
}

impl<'a, T: Component> PtrReader<'a> for &'a T
where
    Self: 'a,
{
    type Item = T;

    fn offset(ptr: NonNull<Self::Item>, bundle: usize) -> Self {
        unsafe { &*ptr.as_ptr().add(bundle) }
    }

    fn access() -> LayoutAccess {
        LayoutAccess(registry::mask::<Self::Item>(), Mask::zero())
    }

    fn fetch(archetype: &Archetype) -> NonNull<Self::Item> {
        let mask = registry::mask::<Self::Item>();
        archetype.vectors[&mask].get_storage_ptr().cast()
    }
}

impl<'a, T: Component> PtrReader<'a> for &'a mut T
where
    Self: 'a,
{
    type Item = T;

    fn offset(ptr: NonNull<Self::Item>, bundle: usize) -> Self {
        unsafe { &mut *ptr.as_ptr().add(bundle) }
    }

    fn access() -> LayoutAccess {
        let mask = registry::mask::<Self::Item>();
        LayoutAccess(mask, mask)
    }

    fn fetch(archetype: &Archetype) -> NonNull<Self::Item> {
        let mask = registry::mask::<Self::Item>();
        archetype.vectors[&mask].get_storage_ptr().cast()
    }
}

impl<'a> PtrReader<'a> for &'a Entity
where
    Self: 'a,
{
    type Item = Entity;

    fn offset(ptr: NonNull<Self::Item>, bundle: usize) -> Self {
        unsafe { &*ptr.as_ptr().add(bundle) }
    }

    fn access() -> LayoutAccess {
        LayoutAccess::none()
    }

    fn fetch(archetype: &Archetype) -> NonNull<Self::Item> {
        // Idk if this is UB but it works fine
        NonNull::new(archetype.entities.as_ptr() as *mut Entity).unwrap()
    }
}
