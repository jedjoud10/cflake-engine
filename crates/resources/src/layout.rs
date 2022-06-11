use ahash::AHashSet;
use std::{
    any::{type_name, TypeId},
    borrow::Borrow,
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    ptr::NonNull, marker::PhantomData,
};

use crate::{Resource, ResourceError, ResourceSet};

// We store the type ID and name in their own struct since the handle might not even be mutable
pub type HandleID = (TypeId, &'static str, bool);

// Resource fetchers are just references to resources, like &mut T or Option<&mut T>
pub trait ResHandle<'a>: Sized {
    type Inner: Resource;
    type Output: 'a;
    const MUTABLE: bool;

    // Get the type ID of the iunner resource
    fn id() -> HandleID {
        (
            TypeId::of::<Self::Inner>(),
            type_name::<Self::Inner>(),
            Self::MUTABLE,
        )
    }

    // Get the underlying pointer for the raw data
    fn fetch_ptr(set: &mut ResourceSet) -> Result<NonNull<Self::Inner>, ResourceError> {
        set.get_casted::<Self::Inner>()
            .map(|r| NonNull::new(r as *mut Self::Inner).unwrap())
    }

    // Convert the pointer into the proper handle
    unsafe fn cast_unchecked(
        ptr: Result<NonNull<Self::Inner>, ResourceError>,
    ) -> Result<Self::Output, ResourceError>;
}

impl<'a, T: Resource> ResHandle<'a> for &'a T {
    type Inner = T;
    type Output = Self;
    const MUTABLE: bool = false;

    unsafe fn cast_unchecked(
        ptr: Result<NonNull<Self::Inner>, ResourceError>,
    ) -> Result<Self::Output, ResourceError> {
        Ok(&*(ptr?.as_ptr() as *const T))
    }
}

impl<'a, T: Resource> ResHandle<'a> for &'a mut T {
    type Inner = T;
    type Output = Self;
    const MUTABLE: bool = true;

    unsafe fn cast_unchecked(
        ptr: Result<NonNull<Self::Inner>, ResourceError>,
    ) -> Result<Self::Output, ResourceError> {
        Ok(&mut *(ptr?.as_ptr() as *mut T))
    }
}

impl<'a, T: Resource> ResHandle<'a> for Option<&'a T> {
    type Inner = T;
    type Output = Self;
    const MUTABLE: bool = false;

    unsafe fn cast_unchecked(
        ptr: Result<NonNull<Self::Inner>, ResourceError>,
    ) -> Result<Self::Output, ResourceError> {
        let res = ptr.ok().map(|ptr| &*(ptr.as_ptr() as *const T));
        Ok(res)
    }
}

impl<'a, T: Resource> ResHandle<'a> for Option<&'a mut T> {
    type Inner = T;
    const MUTABLE: bool = true;
    type Output = Self;

    unsafe fn cast_unchecked(
        ptr: Result<NonNull<Self::Inner>, ResourceError>,
    ) -> Result<Self::Output, ResourceError> {
        let res = ptr.ok().map(|ptr| &mut *(ptr.as_ptr() as *mut T));
        Ok(res)
    }
}

// This implies that the underlying resource MUST exist within the set
// If the resource does not exist, it will simply create it
pub struct AutoInsert<T>(PhantomData<T>);

impl<'a, T: Resource + Default> ResHandle<'a> for AutoInsert<&'a T> {
    type Inner = T;
    const MUTABLE: bool = false;
    type Output = &'a T;

    fn fetch_ptr(set: &mut ResourceSet) -> Result<NonNull<Self::Inner>, ResourceError> {
        if !set.contains::<T>() {
            set.insert(T::default());
        }

        set.get_casted::<Self::Inner>()
            .map(|r| NonNull::new(r as *mut Self::Inner).unwrap())
    }

    unsafe fn cast_unchecked(
        ptr: Result<NonNull<Self::Inner>, ResourceError>,
    ) -> Result<Self::Output, ResourceError> {
        let res = ptr.ok().map(|ptr| &*(ptr.as_ptr() as *const T));
        Ok(res.unwrap())
    }
}


// Bwuh ptr
type Ptr<T> = Result<NonNull<T>, ResourceError>;

// A layout simply multiple resource handles of different resources
pub trait Layout<'a>: Sized {
    type Output: 'a;

    // Get a list of the Handle IDs of the underlying resources
    fn types() -> Vec<HandleID>;

    // Check if the layout is valid (no intersecting handles)
    fn validate() -> Result<(), ResourceError> {
        let types = Self::types();
        let mut map = AHashSet::new();
        let name = types
            .iter()
            .find(|(t, _, mutable)| !map.insert(t) && *mutable);

        // This is a certified inversion classic
        if let Some((_, name, _)) = name {
            Err(ResourceError::Overlapping(name))
        } else {
            Ok(())
        }
    }

    // Get the layout tuple from the resource set without actually checking if the layout is valid
    unsafe fn fetch_unchecked(set: &'a mut ResourceSet) -> Result<Self::Output, ResourceError>;
}

// Simple wrapping function that just gets the handle from the set, and makes it so the lifetime of the handle is different than the one of the set
unsafe fn fetch<'a, A: ResHandle<'a>>(set: &mut ResourceSet) -> Result<A::Output, ResourceError> {
    A::cast_unchecked(A::fetch_ptr(set))
}

impl<'a, A: ResHandle<'a>> Layout<'a> for A {
    type Output = A::Output;

    fn types() -> Vec<HandleID> {
        vec![A::id()]
    }

    unsafe fn fetch_unchecked(set: &'a mut ResourceSet) -> Result<Self::Output, ResourceError> {
        fetch::<A>(set)
    }
}

impl<'a, A: ResHandle<'a>, B: ResHandle<'a>> Layout<'a> for (A, B) {
    type Output = (A::Output, B::Output);

    fn types() -> Vec<HandleID> {
        vec![A::id(), B::id()]
    }

    unsafe fn fetch_unchecked(set: &'a mut ResourceSet) -> Result<Self::Output, ResourceError> {
        Ok((fetch::<A>(set)?, fetch::<B>(set)?))
    }
}

impl<'a, A: ResHandle<'a>, B: ResHandle<'a>, C: ResHandle<'a>> Layout<'a> for (A, B, C) {
    type Output = (A::Output, B::Output, C::Output);

    fn types() -> Vec<HandleID> {
        vec![A::id(), B::id(), C::id()]
    }

    unsafe fn fetch_unchecked(set: &'a mut ResourceSet) -> Result<Self::Output, ResourceError> {
        Ok((fetch::<A>(set)?, fetch::<B>(set)?, fetch::<C>(set)?))
    }
}

impl<'a, A: ResHandle<'a>, B: ResHandle<'a>, C: ResHandle<'a>, D: ResHandle<'a>> Layout<'a>
    for (A, B, C, D)
{
    type Output = (A::Output, B::Output, C::Output, D::Output);

    fn types() -> Vec<HandleID> {
        vec![A::id(), B::id(), C::id(), D::id()]
    }

    unsafe fn fetch_unchecked(set: &'a mut ResourceSet) -> Result<Self::Output, ResourceError> {
        Ok((
            fetch::<A>(set)?,
            fetch::<B>(set)?,
            fetch::<C>(set)?,
            fetch::<D>(set)?,
        ))
    }
}

impl<
        'a,
        A: ResHandle<'a>,
        B: ResHandle<'a>,
        C: ResHandle<'a>,
        D: ResHandle<'a>,
        E: ResHandle<'a>,
    > Layout<'a> for (A, B, C, D, E)
{
    type Output = (A::Output, B::Output, C::Output, D::Output, E::Output);
   
    fn types() -> Vec<HandleID> {
        vec![A::id(), B::id(), C::id(), D::id(), E::id()]
    }

    unsafe fn fetch_unchecked(set: &'a mut ResourceSet) -> Result<Self::Output, ResourceError> {
        Ok((
            fetch::<A>(set)?,
            fetch::<B>(set)?,
            fetch::<C>(set)?,
            fetch::<D>(set)?,
            fetch::<E>(set)?,
        ))
    }
}

impl<
        'a,
        A: ResHandle<'a>,
        B: ResHandle<'a>,
        C: ResHandle<'a>,
        D: ResHandle<'a>,
        E: ResHandle<'a>,
        F: ResHandle<'a>,
    > Layout<'a> for (A, B, C, D, E, F)
{
    type Output = (A::Output, B::Output, C::Output, D::Output, E::Output, F::Output);

    fn types() -> Vec<HandleID> {
        vec![A::id(), B::id(), C::id(), D::id(), E::id(), F::id()]
    }

    unsafe fn fetch_unchecked(set: &'a mut ResourceSet) -> Result<Self::Output, ResourceError> {
        Ok((
            fetch::<A>(set)?,
            fetch::<B>(set)?,
            fetch::<C>(set)?,
            fetch::<D>(set)?,
            fetch::<E>(set)?,
            fetch::<F>(set)?,
        ))
    }
}

impl<
        'a,
        A: ResHandle<'a>,
        B: ResHandle<'a>,
        C: ResHandle<'a>,
        D: ResHandle<'a>,
        E: ResHandle<'a>,
        F: ResHandle<'a>,
        G: ResHandle<'a>,
    > Layout<'a> for (A, B, C, D, E, F, G)
{
    type Output = (A::Output, B::Output, C::Output, D::Output, E::Output, F::Output, G::Output);

    fn types() -> Vec<HandleID> {
        vec![
            A::id(),
            B::id(),
            C::id(),
            D::id(),
            E::id(),
            F::id(),
            G::id(),
        ]
    }

    unsafe fn fetch_unchecked(set: &'a mut ResourceSet) -> Result<Self::Output, ResourceError> {
        Ok((
            fetch::<A>(set)?,
            fetch::<B>(set)?,
            fetch::<C>(set)?,
            fetch::<D>(set)?,
            fetch::<E>(set)?,
            fetch::<F>(set)?,
            fetch::<G>(set)?,
        ))
    }
}

impl<
        'a,
        A: ResHandle<'a>,
        B: ResHandle<'a>,
        C: ResHandle<'a>,
        D: ResHandle<'a>,
        E: ResHandle<'a>,
        F: ResHandle<'a>,
        G: ResHandle<'a>,
        H: ResHandle<'a>,
    > Layout<'a> for (A, B, C, D, E, F, G, H)
{
    type Output = (A::Output, B::Output, C::Output, D::Output, E::Output, F::Output, G::Output, H::Output);

    fn types() -> Vec<HandleID> {
        vec![
            A::id(),
            B::id(),
            C::id(),
            D::id(),
            E::id(),
            F::id(),
            G::id(),
            H::id(),
        ]
    }

    unsafe fn fetch_unchecked(set: &'a mut ResourceSet) -> Result<Self::Output, ResourceError> {
        Ok((
            fetch::<A>(set)?,
            fetch::<B>(set)?,
            fetch::<C>(set)?,
            fetch::<D>(set)?,
            fetch::<E>(set)?,
            fetch::<F>(set)?,
            fetch::<G>(set)?,
            fetch::<H>(set)?,
        ))
    }
}
