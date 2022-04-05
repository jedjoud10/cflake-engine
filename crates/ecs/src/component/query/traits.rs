use std::{mem::ManuallyDrop, cell::UnsafeCell, iter::FlatMap, slice::Iter};
use itertools::{izip, Itertools};
use smallvec::SmallVec;
use crate::{Component, Entity, Mask, Query, QueryError, ComponentError, registry, Archetype};

// Something that can be queried. This will be implement on &T and &mut T (where T is Component). This will also be implemented on &Entity and &BundleData
pub trait QueryItem<'a>: Sized {
    // Create a new iterator out of an archetype
    type Output: Iterator;
    fn archetype_into_iter(archetype: &'a Archetype) -> Self::Output;
}

// QueryItem implementations
impl<'a, T: Component> QueryItem<'a> for &T {
    type Output = std::iter::Map<std::slice::Iter<'a, UnsafeCell<T>>, fn(&UnsafeCell<T>) -> Self>;
    fn archetype_into_iter(archetype: &'a Archetype) -> Self::Output {
        // This result can be cached, but idk how
        let mask = registry::mask::<T>().unwrap();
        let vec = archetype.vectors().get(&mask).unwrap();
        let vec = vec.as_any().downcast_ref::<Vec<UnsafeCell<T>>>().unwrap();
        vec.iter().map(|cell| unsafe { &*cell.get() })
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
impl<'a, T: Component> QueryItem<'a> for &mut T {
    type Output = std::iter::Map<std::slice::Iter<'a, UnsafeCell<T>>, fn(&UnsafeCell<T>) -> Self>;
    fn archetype_into_iter(archetype: &'a Archetype) -> Self::Output {
        // This result can be cached, but idk how
        let mask = registry::mask::<T>().unwrap();
        let vec = archetype.vectors().get(&mask).unwrap();
        let vec = vec.as_any().downcast_ref::<Vec<UnsafeCell<T>>>().unwrap();
        vec.iter().map(|cell| unsafe { &mut *cell.get() })
    }
}
impl<'a> QueryItem<'a> for &Entity {
    type Output = std::iter::FilterMap<std::slice::Iter<'a, (Entity, bool)>, fn(&(Entity, bool)) -> Option<Entity>>;
    fn archetype_into_iter(archetype: &'a Archetype) -> Self::Output {
        archetype.entities().iter().filter_map(|(entity, pending_for_removal)| bool::then_some(*pending_for_removal, *entity))
    }
}

// Layout query that contains multiple QueryItems
pub trait LayoutQuery: Sized {
    // Calculate the mask using the current layout
    fn mask() -> Result<Mask, ComponentError>;
    // Create a query using the mask
    fn query(query: &Query<impl LayoutQuery>) -> Result<Vec<Self>, QueryError>;
}

// Convert an iterator into the a properly sized vector
fn into_vec<Item>(num: usize, iter: impl Iterator<Item = Item>) -> Vec<Item> {
    let mut vec = Vec::<Item>::with_capacity(num);
    vec.extend(iter);
    vec
}

// LayoutQuery implementations
// This could really use some macro magic, though I have no idea how I would make it work
/*
impl<A: QueryItem> LayoutQuery for A {
    fn mask() -> Result<Mask, ComponentError> {
        //registry::mask::<A::Component>()
        todo!()
    }

    fn query(query: &Query<impl LayoutQuery>) -> Result<Vec<Self>, QueryError> {
        //let a = query.get_cells()?.map(A::convert);
        //Ok(into_vec(query.count(), a))
        todo!()
    }
}
*/
/*
impl<A: QueryItem, B: QueryItem> LayoutQuery for (A, B) {
    fn mask() -> Result<Mask, ComponentError> {
        Ok(registry::mask::<A::Component>()? | registry::mask::<B::Component>()?)
    }

    fn query(query: &Query<impl LayoutQuery>) -> Result<Vec<Self>, QueryError> {
        let a = query.get_cells()?.map(A::convert);
        let b = query.get_cells()?.map(B::convert);
        let zipped = izip!(a, b);
        Ok(into_vec(query.count(), zipped))
    }
}
impl<A: QueryItem, B: QueryItem, C: QueryItem> LayoutQuery for (A, B, C) {
    fn mask() -> Result<Mask, ComponentError> {
        Ok(registry::mask::<A::Component>()? | registry::mask::<B::Component>()? | registry::mask::<C::Component>()?)
    }

    fn query(query: &Query<impl LayoutQuery>) -> Result<Vec<Self>, QueryError> {
        let a = query.get_cells()?.map(A::convert);
        let b = query.get_cells()?.map(B::convert);
        let c = query.get_cells()?.map(C::convert);
        let zipped = izip!(a, b, c);
        Ok(into_vec(query.count(), zipped))
    }
}
*/