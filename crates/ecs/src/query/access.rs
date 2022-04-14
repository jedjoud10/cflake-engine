use crate::{registry, Component, Mask, QueryError};
use std::ptr::NonNull;
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

    // Get the normal component mask AND writing mask
    fn mask() -> Result<(Mask, Mask), QueryError> {
        // Get the normal mask
        let mask = registry::mask::<Self::Component>().map_err(QueryError::ComponentError)?;

        // Get the writing mask
        let writing = Self::WRITING_MASK_ENABLED.then(|| mask).unwrap_or_default();

        Ok((mask, writing))
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
    const WRITING_MASK_ENABLED: bool = true;

    fn offset(ptr: NonNull<Self::Component>, bundle: usize) -> Self::Borrowed {
        unsafe { &mut *ptr.as_ptr().add(bundle) }
    }
}
