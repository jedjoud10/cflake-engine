use std::{mem::ManuallyDrop, cell::UnsafeCell};
use itertools::izip;
use smallvec::SmallVec;
use crate::{registry, Component, Entity, Mask, Query, QueryError, ARCHETYPE_INLINE_SIZE, ComponentError};

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

    #[inline(always)]
    fn convert(cell: &UnsafeCell<Self::Component>) -> Self { unsafe { &*cell.get() }}
}
impl<T: Component> QueryItem for &mut T {
    type Component = T;

    #[inline(always)]
    fn convert(cell: &UnsafeCell<Self::Component>) -> Self { unsafe { &mut *cell.get() }}
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
pub trait LayoutQuery: Sized {
    // Calculate the mask using the current layout
    fn mask() -> Result<Mask, ComponentError>;
    // Create a query using the mask
    fn query(query: &Query<impl LayoutQuery>) -> Result<Vec<Self>, QueryError>;
}

// LayoutQuery implementations
// This could really use some macro magic, though I have no idea how I would make it work
impl<A: QueryItem> LayoutQuery for A {
    fn mask() -> Result<Mask, ComponentError> {
        registry::mask::<A::Component>()
    }

    fn query(query: &Query<impl LayoutQuery>) -> Result<Vec<Self>, QueryError> {
        let a = query.get_cells()?.map(A::convert);
        Ok(a.collect::<Vec<_>>())
    }
}
impl<A: QueryItem, B: QueryItem> LayoutQuery for (A, B) {
    fn mask() -> Result<Mask, ComponentError> {
        Ok(registry::mask::<A::Component>()? | registry::mask::<B::Component>()?)
    }

    fn query(query: &Query<impl LayoutQuery>) -> Result<Vec<Self>, QueryError> {
        let a = query.get_cells()?.map(A::convert);
        let b = query.get_cells()?.map(B::convert);
        Ok(izip!(a, b).collect::<Vec<_>>())
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
        //zipped.for_each(|(a, b, c)| {});
        let i = std::time::Instant::now();
        let res = Ok(zipped.collect::<Vec<_>>());
        dbg!(i.elapsed());
        res
    }
}