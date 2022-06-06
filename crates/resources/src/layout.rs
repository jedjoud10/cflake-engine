use std::{any::TypeId, cell::{Ref, RefMut, RefCell}, borrow::Borrow, ptr::NonNull, collections::HashMap};
use ahash::AHashSet;

use crate::{Resource, ResourceSet, ResourceError};

// Resource fetchers are just references to resources, like &mut T or Option<&mut T>
pub trait ResHandle<'a>: Sized {
    type Inner: Resource;

    // Get the type ID of the iunner resource
    fn id() -> TypeId {
        TypeId::of::<Self::Inner>()
    }

    // Get the underlying pointer for the raw data
    fn fetch_ptr(set: &mut ResourceSet) -> Result<NonNull<Self::Inner>, ResourceError> {
        let boxed = set.get_mut(&Self::id()).ok_or(ResourceError::missing::<Self::Inner>())?;
        let ptr = boxed.as_any_mut().downcast_mut::<Self::Inner>().unwrap() as *mut Self::Inner;
        Ok(NonNull::new(ptr).unwrap())
    }

    // Convert the pointer into the proper handle
    unsafe fn cast_unchecked(ptr: Result<NonNull<Self::Inner>, ResourceError>) -> Result<Self, ResourceError>;
}

impl<'a, T: Resource> ResHandle<'a> for &'a T {
    type Inner = T;

    unsafe fn cast_unchecked(ptr: Result<NonNull<Self::Inner>, ResourceError>) -> Result<Self, ResourceError> {
        Ok(&*(ptr?.as_ptr() as *const T))
    }
}

impl<'a, T: Resource> ResHandle<'a> for &'a mut T {
    type Inner = T;

    unsafe fn cast_unchecked(ptr: Result<NonNull<Self::Inner>, ResourceError>) -> Result<Self, ResourceError> {
        Ok(&mut *(ptr?.as_ptr() as *mut T))
    }
}

impl<'a, T: Resource> ResHandle<'a> for Option<&'a T> {
    type Inner = T;

    unsafe fn cast_unchecked(ptr: Result<NonNull<Self::Inner>, ResourceError>) -> Result<Self, ResourceError> {
        let res = ptr.ok().map(|ptr| &*(ptr.as_ptr() as *const T));
        Ok(res)
    }
}

impl<'a, T: Resource> ResHandle<'a> for Option<&'a mut T> {
    type Inner = T;

    unsafe fn cast_unchecked(ptr: Result<NonNull<Self::Inner>, ResourceError>) -> Result<Self, ResourceError> {
        let res = ptr.ok().map(|ptr| &mut *(ptr.as_ptr() as *mut T));
        Ok(res)
    }
}

// Bwuh
type Ptr<T> = Result<NonNull<T>, ResourceError>;

// A layout simply multiple resource handles of different resources
pub trait Layout<'a>: Sized {
    // Get a list of the TypeIDs of the underlying resources
    fn types() -> Vec<TypeId>;

    // Check if the layout is valid (no intersecting handles)
    fn is_valid() -> bool {
        let mut map = AHashSet::new();
        Self::types().into_iter().all(|t| map.insert(t))
    }

    // Get the layout tuple from the resource set without actually checking if the layout is valid
    unsafe fn fetch_unchecked(set: &'a mut ResourceSet) -> Result<Self, ResourceError>;
}

// Simple wrapping function that just gets the handle from the set, and makes it so the lifetime of the handle is different than the one of the set
unsafe fn fetch<'a, A: ResHandle<'a>>(set: &mut ResourceSet) -> Result<A, ResourceError> {
    A::cast_unchecked(A::fetch_ptr(set))
}

impl<'a, A: ResHandle<'a>> Layout<'a> for A {
    fn types() -> Vec<TypeId> {
        vec![A::id()]
    }

    unsafe fn fetch_unchecked(set: &'a mut ResourceSet) -> Result<Self, ResourceError> {
        fetch(set)
    }
}

impl<'a, A: ResHandle<'a>, B: ResHandle<'a>> Layout<'a> for (A, B) {
    fn types() -> Vec<TypeId> {
        vec![A::id(), B::id()]
    }

    unsafe fn fetch_unchecked(set: &'a mut ResourceSet) -> Result<Self, ResourceError> {
        Ok((fetch::<A>(set)?, fetch::<B>(set)?))
    }
}

impl<'a, A: ResHandle<'a>, B: ResHandle<'a>, C: ResHandle<'a>> Layout<'a> for (A, B, C) {
    fn types() -> Vec<TypeId> {
        vec![A::id(), B::id(), C::id()]
    }

    unsafe fn fetch_unchecked(set: &'a mut ResourceSet) -> Result<Self, ResourceError> {
        Ok((fetch::<A>(set)?, fetch::<B>(set)?, fetch::<C>(set)?))
    }
}

impl<'a, A: ResHandle<'a>, B: ResHandle<'a>, C: ResHandle<'a>, D: ResHandle<'a>> Layout<'a> for (A, B, C, D) {
    fn types() -> Vec<TypeId> {
        vec![A::id(), B::id(), C::id(), D::id()]
    }

    unsafe fn fetch_unchecked(set: &'a mut ResourceSet) -> Result<Self, ResourceError> {
        Ok((fetch::<A>(set)?, fetch::<B>(set)?, fetch::<C>(set)?, fetch::<D>(set)?))
    }
}