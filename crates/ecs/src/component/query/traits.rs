use std::{mem::ManuallyDrop, cell::UnsafeCell};
use itertools::izip;
use smallvec::SmallVec;
use crate::{registry, Component, Entity, Mask, Query, QueryError, ARCHETYPE_INLINE_SIZE, ComponentError, InlinedStorages};

// Something that can be queried. This will be implement on &T and &mut T (where T is Component)
pub trait QueryItem: Sized {
    // Returns the underlying storages
    type Component: Component;
    // Convert an unsafe cell to it's reference (either mutable or immutable)
    fn convert(cell: &UnsafeCell<Self::Component>) -> Self;
}

// QueryItem implementations
impl<T: Component> QueryItem for &T {
    type Component = T;

    fn convert(cell: &UnsafeCell<Self::Component>) -> Self {
        unsafe { &*cell.get() }
    }
}
impl<T: Component> QueryItem for &mut T {
    type Component = T;

    fn convert(cell: &UnsafeCell<Self::Component>) -> Self {
        unsafe { &mut *cell.get() }
    }
}
/*
impl QueryItem for Entity {
    fn create_query_vec<Layout: LayoutQuery>(query: &Query<Layout>) -> Result<Vec<Self>, QueryError>
    {
        let entities = query.filter_archetypes().flat_map(|archetype| archetype.entities()).cloned();
        Ok(entities.collect::<Vec<_>>())
    }

    fn try_get_mask() -> Option<Mask> { None }
}
*/

// Layout query that contains multiple QueryItems
pub trait LayoutQuery {
    type Layout;
    // Calculate the mask using the current layout
    fn mask() -> Result<Mask, ComponentError>;
    // Get the layout query's query
    fn query<Layout: LayoutQuery>(query: &Query<Layout>) -> Result<Vec<Self::Layout>, QueryError>;
}

// LayoutQuery implementations
// This could really use some macro magic, though I have no idea how I would make it work
impl<A: QueryItem> LayoutQuery for A {
    type Layout = A;

    fn mask() -> Result<Mask, ComponentError> {
        registry::mask::<A::Component>()
    }

    fn query<Layout: LayoutQuery>(query: &Query<Layout>) -> Result<Vec<Self::Layout>, QueryError> {
        let vec: InlinedStorages<A::Component> = query.get_storages::<A::Component>();
        let cells = vec.into_iter().flat_map(|storage| storage.iter());
        let vars = cells.map(A::convert).collect::<Vec<A>>();
        Ok(vars)
    }
}
impl<A: QueryItem, B: QueryItem> LayoutQuery for (A, B) {
    type Layout = (A, B);

    fn mask() -> Result<Mask, ComponentError> {
        Ok(registry::mask::<A::Component>()? | registry::mask::<B::Component>()?)
    }

    fn query<Layout: LayoutQuery>(query: &Query<Layout>) -> Result<Vec<Self::Layout>, QueryError> {
        let vec_a = query.get_storages::<A::Component>().into_iter().flat_map(|s| s.iter());
        let vec_b = query.get_storages::<B::Component>().into_iter().flat_map(|s| s.iter());
        let zipped = izip!(vec_a, vec_b);
        let vars = zipped.map(|(a, b)| (A::convert(a), B::convert(b))).collect::<Vec<(A, B)>>();
        Ok(vars)
    }
}
impl<A: QueryItem, B: QueryItem, C: QueryItem> LayoutQuery for (A, B, C) {
    type Layout = (A, B, C);

    fn mask() -> Result<Mask, ComponentError> {
        Ok(registry::mask::<A::Component>()? | registry::mask::<B::Component>()? | registry::mask::<C::Component>()?)
    }

    fn query<Layout: LayoutQuery>(query: &Query<Layout>) -> Result<Vec<Self::Layout>, QueryError> {
        let vec_a = query.get_storages::<A::Component>().into_iter().flat_map(|s| s.iter());
        let vec_b = query.get_storages::<B::Component>().into_iter().flat_map(|s| s.iter());
        let vec_c = query.get_storages::<C::Component>().into_iter().flat_map(|s| s.iter());
        let zipped = izip!(vec_a, vec_b, vec_c);
        let vars = zipped.map(|(a, b, c)| (A::convert(a), B::convert(b), C::convert(c))).collect::<Vec<_>>();
        Ok(vars)
    }
}