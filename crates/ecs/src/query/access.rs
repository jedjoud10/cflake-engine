use crate::{registry, Component, ComponentStateRow, ComponentStateSet, Mask};
use std::{ops::BitOr, ptr::NonNull};

// Layout access that contain the normal mask and writing mask
#[derive(Clone, Copy)]
pub struct LayoutAccess(Mask, Mask);

impl LayoutAccess {
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

// Trait that will be implmenented for &T and &'a T where T is a component
pub trait PtrReader<'a> {
    type Component: Component;
    const WRITING_MASK_ENABLED: bool;

    // Offset an unsafe pointer by a bundle offset and read it
    fn offset(ptr: NonNull<Self::Component>, bundle: usize) -> Self;

    // Get the normal component mask AND writing mask, combined into a single layout mask
    fn access() -> LayoutAccess {
        // Get the normal mask
        let mask = registry::mask::<Self::Component>();

        // Get the writing mask
        let writing = Self::WRITING_MASK_ENABLED.then(|| mask).unwrap_or_else(Mask::zero);

        LayoutAccess(mask, writing)
    }
}

impl<'a, T: Component> PtrReader<'a> for &'a T
where
    Self: 'a,
{
    type Component = T;
    const WRITING_MASK_ENABLED: bool = false;

    fn offset(ptr: NonNull<Self::Component>, bundle: usize) -> Self {
        unsafe { &*ptr.as_ptr().add(bundle) }
    }
}

impl<'a, T: Component> PtrReader<'a> for &'a mut T
where
    Self: 'a,
{
    type Component = T;
    const WRITING_MASK_ENABLED: bool = true;

    fn offset(ptr: NonNull<Self::Component>, bundle: usize) -> Self {
        unsafe { &mut *ptr.as_ptr().add(bundle) }
    }
}
