use crate::{registry, Component, Mask};
use std::{ops::BitOr, ptr::NonNull};

// Layout access that contain the normal mask and writing mask
#[derive(Clone, Copy)]
pub struct LayoutAccess(Mask, Mask);

impl LayoutAccess {
    // Get the normal mask
    pub fn reading(&self) -> &Mask {
        &self.0
    }
    // Get the writing mask
    pub fn writing(&self) -> &Mask {
        &self.1
    }
}

impl BitOr for LayoutAccess {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0, self.1 | rhs.1)
    }
}

// Gets a "&" reference to the component
pub struct Read<T: 'static + Component>(&'static T);

// Gets a "&mut" reference to the component
pub struct Write<T: 'static + Component, const SILENT: bool = false>(&'static mut T);

// Trait that will be implmenented for Read<T> and Write<T>
pub trait PtrReader<'a> {
    type Component: 'static + Component;
    type Borrowed: 'a;
    const WRITING_MASK_ENABLED: bool;

    // Offset an unsafe pointer by a bundle offset and read it
    fn offset(ptr: NonNull<Self::Component>, bundle: usize) -> Self::Borrowed;

    // Get the normal component mask AND writing mask, combined into a single layout mask
    fn access() -> LayoutAccess {
        // Get the normal mask
        let mask = registry::mask::<Self::Component>();

        // Get the writing mask
        let writing = Self::WRITING_MASK_ENABLED.then(|| mask).unwrap_or_else(Mask::zero);

        LayoutAccess(mask, writing)
    }
}

impl<'a, T: Component> PtrReader<'a> for Read<T>
where
    Self: 'a,
{
    type Component = T;
    type Borrowed = &'a T;
    const WRITING_MASK_ENABLED: bool = false;

    fn offset(ptr: NonNull<Self::Component>, bundle: usize) -> Self::Borrowed {
        unsafe { &*ptr.as_ptr().add(bundle) }
    }
}

impl<'a, T: Component, const SILENT: bool> PtrReader<'a> for Write<T, SILENT>
where
    Self: 'a,
{
    type Component = T;
    type Borrowed = &'a mut T;
    const WRITING_MASK_ENABLED: bool = !SILENT;

    fn offset(ptr: NonNull<Self::Component>, bundle: usize) -> Self::Borrowed {
        unsafe { &mut *ptr.as_ptr().add(bundle) }
    }
}
