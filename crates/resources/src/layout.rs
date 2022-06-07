use ahash::AHashSet;
use std::{
    any::{type_name, TypeId},
    borrow::Borrow,
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    ptr::NonNull,
};

use crate::{Resource, ResourceError, ResourceSet};

// Resource fetchers are just references to resources, like &mut T or Option<&mut T>
pub trait ResHandle<'a>: Sized {
    type Inner: Resource;

    // Get the type ID of the iunner resource
    fn id() -> (TypeId, &'static str) {
        (TypeId::of::<Self::Inner>(), type_name::<Self::Inner>())
    }

    // Get the underlying pointer for the raw data
    fn fetch_ptr(set: &mut ResourceSet) -> Result<NonNull<Self::Inner>, ResourceError> {
        set.get_casted::<Self::Inner>().map(|r| NonNull::new(r as *mut Self::Inner).unwrap())
    }

    // Convert the pointer into the proper handle
    unsafe fn cast_unchecked(
        ptr: Result<NonNull<Self::Inner>, ResourceError>,
    ) -> Result<Self, ResourceError>;
}

impl<'a, T: Resource> ResHandle<'a> for &'a T {
    type Inner = T;

    unsafe fn cast_unchecked(
        ptr: Result<NonNull<Self::Inner>, ResourceError>,
    ) -> Result<Self, ResourceError> {
        Ok(&*(ptr?.as_ptr() as *const T))
    }
}

impl<'a, T: Resource> ResHandle<'a> for &'a mut T {
    type Inner = T;

    unsafe fn cast_unchecked(
        ptr: Result<NonNull<Self::Inner>, ResourceError>,
    ) -> Result<Self, ResourceError> {
        Ok(&mut *(ptr?.as_ptr() as *mut T))
    }
}

impl<'a, T: Resource> ResHandle<'a> for Option<&'a T> {
    type Inner = T;

    unsafe fn cast_unchecked(
        ptr: Result<NonNull<Self::Inner>, ResourceError>,
    ) -> Result<Self, ResourceError> {
        let res = ptr.ok().map(|ptr| &*(ptr.as_ptr() as *const T));
        Ok(res)
    }
}

impl<'a, T: Resource> ResHandle<'a> for Option<&'a mut T> {
    type Inner = T;

    unsafe fn cast_unchecked(
        ptr: Result<NonNull<Self::Inner>, ResourceError>,
    ) -> Result<Self, ResourceError> {
        let res = ptr.ok().map(|ptr| &mut *(ptr.as_ptr() as *mut T));
        Ok(res)
    }
}

// Bwuh
type Ptr<T> = Result<NonNull<T>, ResourceError>;

// A layout simply multiple resource handles of different resources
pub trait Layout<'a>: Sized {
    // Get a list of the TypeIDs of the underlying resources
    fn types() -> Vec<(TypeId, &'static str)>;

    // Check if the layout is valid (no intersecting handles)
    fn validate() -> Result<(), ResourceError> {
        let types = Self::types();
        let mut map = AHashSet::new();
        let name = types.iter().find(|(t, _)| !map.insert(t));

        // This is a certified inversion classic
        if let Some((_, name)) = name {
            Err(ResourceError::Overlapping(name))
        } else {
            Ok(())
        }
    }

    // Get the layout tuple from the resource set without actually checking if the layout is valid
    unsafe fn fetch_unchecked(set: &'a mut ResourceSet) -> Result<Self, ResourceError>;
}

// Simple wrapping function that just gets the handle from the set, and makes it so the lifetime of the handle is different than the one of the set
unsafe fn fetch<'a, A: ResHandle<'a>>(set: &mut ResourceSet) -> Result<A, ResourceError> {
    A::cast_unchecked(A::fetch_ptr(set))
}

impl<'a, A: ResHandle<'a>> Layout<'a> for A {
    fn types() -> Vec<(TypeId, &'static str)> {
        vec![A::id()]
    }

    unsafe fn fetch_unchecked(set: &'a mut ResourceSet) -> Result<Self, ResourceError> {
        fetch(set)
    }
}

impl<'a, A: ResHandle<'a>, B: ResHandle<'a>> Layout<'a> for (A, B) {
    fn types() -> Vec<(TypeId, &'static str)> {
        vec![A::id(), B::id()]
    }

    unsafe fn fetch_unchecked(set: &'a mut ResourceSet) -> Result<Self, ResourceError> {
        Ok((fetch::<A>(set)?, fetch::<B>(set)?))
    }
}

impl<'a, A: ResHandle<'a>, B: ResHandle<'a>, C: ResHandle<'a>> Layout<'a> for (A, B, C) {
    fn types() -> Vec<(TypeId, &'static str)> {
        vec![A::id(), B::id(), C::id()]
    }

    unsafe fn fetch_unchecked(set: &'a mut ResourceSet) -> Result<Self, ResourceError> {
        Ok((fetch::<A>(set)?, fetch::<B>(set)?, fetch::<C>(set)?))
    }
}

impl<'a, A: ResHandle<'a>, B: ResHandle<'a>, C: ResHandle<'a>, D: ResHandle<'a>> Layout<'a>
    for (A, B, C, D)
{
    fn types() -> Vec<(TypeId, &'static str)> {
        vec![A::id(), B::id(), C::id(), D::id()]
    }

    unsafe fn fetch_unchecked(set: &'a mut ResourceSet) -> Result<Self, ResourceError> {
        Ok((
            fetch::<A>(set)?,
            fetch::<B>(set)?,
            fetch::<C>(set)?,
            fetch::<D>(set)?,
        ))
    }
}