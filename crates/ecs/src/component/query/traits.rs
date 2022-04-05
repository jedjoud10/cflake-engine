use crate::{registry, Archetype, BundleState, Component, ComponentError, Entity, Mask, Query, QueryError};
use itertools::{izip, Itertools};
use smallvec::SmallVec;
use std::{cell::UnsafeCell, iter::FlatMap, mem::ManuallyDrop, slice::Iter};

// Something that can be queried. This will be implement on &T and &mut T (where T is Component). This will also be implemented on &Entity and &BundleData
pub trait QueryItem<'a>: Sized {
    // Create a new iterator out of an archetype
    type Output: Iterator<Item = Self>;
    fn archetype_into_iter(archetype: &'a Archetype) -> Self::Output;
    // Get mask influence
    fn try_get_mask() -> Result<Mask, ComponentError>;
}

// QueryItem implementations
impl<'a, T: Component> QueryItem<'a> for &'a T {
    type Output = std::iter::Map<std::slice::Iter<'a, UnsafeCell<T>>, fn(&UnsafeCell<T>) -> Self>;
    fn archetype_into_iter(archetype: &'a Archetype) -> Self::Output {
        // This result can be cached, but idk how
        let mask = registry::mask::<T>().unwrap();
        let vec = archetype.vectors().get(&mask).unwrap();
        let vec = vec.as_any().downcast_ref::<Vec<UnsafeCell<T>>>().unwrap();
        vec.iter().map(|cell| unsafe { &*cell.get() })
    }
    fn try_get_mask() -> Result<Mask, ComponentError> {
        registry::mask::<T>()
    }
    /*
    fn convert<'a, Input: Iterator<Item = &'a Archetype>, Output: Iterator<Item = Self>>(input: Input) -> Output {
        let mask = registry::mask::<T>().unwrap();
        input.flat_map(move |archetype| {
            // Fetch the components
            let vec = archetype.vectors().get(&mask).unwrap();
            let vec = vec.as_any().downcast_ref::<Vec<UnsafeCell<T>>>().unwrap();
            vec.iter()
        })
    }
    */
}
impl<'a, T: Component> QueryItem<'a> for &'a mut T {
    type Output = std::iter::Map<std::slice::Iter<'a, UnsafeCell<T>>, fn(&UnsafeCell<T>) -> Self>;
    fn archetype_into_iter(archetype: &'a Archetype) -> Self::Output {
        // This result can be cached, but idk how
        let mask = registry::mask::<T>().unwrap();
        let vec = archetype.vectors().get(&mask).unwrap();
        let vec = vec.as_any().downcast_ref::<Vec<UnsafeCell<T>>>().unwrap();
        vec.iter().map(|cell| unsafe { &mut *cell.get() })
    }
    fn try_get_mask() -> Result<Mask, ComponentError> {
        registry::mask::<T>()
    }
}
impl<'a> QueryItem<'a> for &'a Entity {
    type Output = std::slice::Iter<'a, Entity>;
    fn archetype_into_iter(archetype: &'a Archetype) -> Self::Output {
        archetype.entities().iter()
    }
    fn try_get_mask() -> Result<Mask, ComponentError> {
        Ok(Mask::default())
    }
}
/*
impl<'a> QueryItem<'a> for &'a BundleState {
    type Output;

    fn archetype_into_iter(archetype: &'a Archetype) -> Self::Output {
        // Get the two states and zip them into a BundleState
        let entities = archetype.states().entities.iter();
        let components = archetype.states().components.get().iter();
        (0..archetype.entities().len()).into_iter().map(|index| {
        })
    }

    fn try_get_mask() -> Result<Mask, ComponentError> { Ok(Mask::default()) }
}
*/
// Layout query that contains multiple QueryItems
pub trait LayoutQuery<'a>: Sized {
    // Calculate the mask using the current layout
    fn mask() -> Result<Mask, ComponentError>;
    // Create a query using the mask
    fn query_from_archetypes(archetypes: impl Iterator<Item = &'a Archetype>, count: usize) -> Result<Vec<Self>, QueryError>;
}

// Convert an iterator into the a properly sized vector
fn into_vec<Item>(num: usize, iter: impl Iterator<Item = Item>) -> Vec<Item> {
    let mut vec = Vec::<Item>::with_capacity(num);
    vec.extend(iter);
    vec
}

// LayoutQuery implementations
// This could really use some macro magic, though I have no idea how I would make it work
impl<'a, A: QueryItem<'a>> LayoutQuery<'a> for A {
    fn mask() -> Result<Mask, ComponentError> {
        A::try_get_mask()
    }

    fn query_from_archetypes(archetypes: impl Iterator<Item = &'a Archetype>, count: usize) -> Result<Vec<Self>, QueryError> {
        Ok(into_vec(count, archetypes.flat_map(|archetype| A::archetype_into_iter(archetype))))
    }
}
impl<'a, A: QueryItem<'a>, B: QueryItem<'a>> LayoutQuery<'a> for (A, B) {
    fn mask() -> Result<Mask, ComponentError> {
        Ok(A::try_get_mask()? | B::try_get_mask()?)
    }

    fn query_from_archetypes(archetypes: impl Iterator<Item = &'a Archetype>, count: usize) -> Result<Vec<Self>, QueryError> {
        Ok(into_vec(
            count,
            archetypes.flat_map(|archetype| izip!(A::archetype_into_iter(archetype), B::archetype_into_iter(archetype))),
        ))
    }
}
impl<'a, A: QueryItem<'a>, B: QueryItem<'a>, C: QueryItem<'a>> LayoutQuery<'a> for (A, B, C) {
    fn mask() -> Result<Mask, ComponentError> {
        Ok(A::try_get_mask()? | B::try_get_mask()? | C::try_get_mask()?)
    }

    fn query_from_archetypes(archetypes: impl Iterator<Item = &'a Archetype>, count: usize) -> Result<Vec<Self>, QueryError> {
        Ok(into_vec(
            count,
            archetypes.flat_map(|archetype| izip!(A::archetype_into_iter(archetype), B::archetype_into_iter(archetype), C::archetype_into_iter(archetype))),
        ))
    }
}
