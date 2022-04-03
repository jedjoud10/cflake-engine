use std::mem::ManuallyDrop;

use crate::{registry, Component, Entity, Mask, Query, QueryError};

// Something that can be queried. This will be implement on &T and &mut T (where T is Component)
pub trait QueryItem: Sized {
    // Fill an iterator of &mut Option<Self> with valid query items that we fetch from the builder
    fn fill_iterator<'a, Layout: LayoutQuery>(builder: &Query<Layout>, iter: impl Iterator<Item = &'a mut Option<Self>>) -> Result<(), QueryError>
    where
        Self: 'a;
    // Try to get the mask of this query item. This might fail in the case of a QueryItem of type Entity
    fn try_get_mask() -> Option<Mask>;
}

// QueryItem implementations
impl<'b, T: Component> QueryItem for &'b T {
    fn fill_iterator<'a, Layout: LayoutQuery>(builder: &Query<Layout>, iter: impl Iterator<Item = &'a mut Option<Self>>) -> Result<(), QueryError>
    where
        'b: 'a,
    {
        let refs = builder.get_component_vec_mapped::<T, &'b T, _>(|cell| unsafe { &*cell.get() })?;
        refs.zip(iter).for_each(|(val, res)| *res = Some(val));
        Ok(())
    }

    fn try_get_mask() -> Option<Mask> { registry::mask::<T>().ok() }
}
impl<'b, T: Component> QueryItem for &'b mut T {
    fn fill_iterator<'a, Layout: LayoutQuery>(builder: &Query<Layout>, iter: impl Iterator<Item = &'a mut Option<Self>>) -> Result<(), QueryError>
    where
        'b: 'a,
    {
        let refs = builder.get_component_vec_mapped::<T, &'b mut T, _>(|cell| unsafe { &mut *cell.get() })?;
        refs.zip(iter).for_each(|(val, res)| *res = Some(val));
        Ok(())
    }

    fn try_get_mask() -> Option<Mask> { registry::mask::<T>().ok() }
}
impl QueryItem for Entity {
    fn fill_iterator<'a, Layout: LayoutQuery>(builder: &Query<Layout>, iter: impl Iterator<Item = &'a mut Option<Self>>) -> Result<(), QueryError> {
        let entities = builder.filter_archetypes().flat_map(|archetype| archetype.entities()).cloned();
        entities.zip(iter).for_each(|(val, res)| *res = Some(val));
        Ok(())
    }

    fn try_get_mask() -> Option<Mask> { None }
}

// Layout query that contains multiple QueryItems
pub trait LayoutQuery {
    type Layout;
    // Calculate the mask using the current layout
    fn mask() -> Mask;
    // Get the layout query's query
    fn query<Layout: LayoutQuery>(builder: &Query<Layout>) -> Result<Vec<Self::Layout>, QueryError>;
}

// Convert a Vec<(Option<A>, Option<B>...)> to Vec<(A, B)>
unsafe fn unwrap_vec<Input, Output>(vec: Vec<Input>) -> Vec<Output> {
    // We are 100% sure that the vector is filled with Some, so we can just transmute it
    let mut manual = ManuallyDrop::new(vec);
    let vec = Vec::from_raw_parts(manual.as_mut_ptr() as *mut Output, manual.len(), manual.capacity());
    ManuallyDrop::drop(&mut manual);
    vec
}

// LayoutQuery implementations
// This could really use some macro magic, though I have no idea how I would make it work
impl<A: QueryItem> LayoutQuery for A {
    type Layout = A;

    fn mask() -> Mask {
        A::try_get_mask().unwrap_or_default()
    }

    fn query<Layout: LayoutQuery>(builder: &Query<Layout>) -> Result<Vec<Self::Layout>, QueryError> {
        let mut vec: Vec<Option<A>> = {
            let mut vec = Vec::new();
            vec.resize_with(builder.get_component_count(), || None);
            vec
        };

        A::fill_iterator(builder, vec.iter_mut())?;
        Ok(unsafe { unwrap_vec(vec) })
    }
}
impl<A: QueryItem, B: QueryItem> LayoutQuery for (A, B) {
    type Layout = (A, B);

    fn mask() -> Mask {
        A::try_get_mask().unwrap_or_default() | B::try_get_mask().unwrap_or_default()
    }

    fn query<Layout: LayoutQuery>(builder: &Query<Layout>) -> Result<Vec<Self::Layout>, QueryError> {
        let mut vec: Vec<(Option<A>, Option<B>)> = {
            let mut vec = Vec::new();
            vec.resize_with(builder.get_component_count(), || (None, None));
            vec
        };

        A::fill_iterator(builder, vec.iter_mut().map(|(a, _)| a))?;
        B::fill_iterator(builder, vec.iter_mut().map(|(_, b)| b))?;
        Ok(unsafe { unwrap_vec(vec) })
    }
}
impl<A: QueryItem, B: QueryItem, C: QueryItem> LayoutQuery for (A, B, C) {
    type Layout = (A, B, C);

    fn mask() -> Mask {
        A::try_get_mask().unwrap_or_default() | B::try_get_mask().unwrap_or_default() | C::try_get_mask().unwrap_or_default()
    }

    fn query<Layout: LayoutQuery>(builder: &Query<Layout>) -> Result<Vec<Self::Layout>, QueryError> {
        let mut vec: Vec<(Option<A>, Option<B>, Option<C>)> = {
            let mut vec = Vec::new();
            vec.resize_with(builder.get_component_count(), || (None, None, None));
            vec
        };

        A::fill_iterator(builder, vec.iter_mut().map(|(a, _, _)| a))?;
        B::fill_iterator(builder, vec.iter_mut().map(|(_, b, _)| b))?;
        C::fill_iterator(builder, vec.iter_mut().map(|(_, _, c)| c))?;
        Ok(unsafe { unwrap_vec(vec) })
    }
}
impl<A: QueryItem, B: QueryItem, C: QueryItem, D: QueryItem> LayoutQuery for (A, B, C, D) {
    type Layout = (A, B, C, D);

    fn mask() -> Mask {
        A::try_get_mask().unwrap_or_default() | B::try_get_mask().unwrap_or_default() | C::try_get_mask().unwrap_or_default() | D::try_get_mask().unwrap_or_default()
    }

    fn query<Layout: LayoutQuery>(builder: &Query<Layout>) -> Result<Vec<Self::Layout>, QueryError> {
        let mut vec: Vec<(Option<A>, Option<B>, Option<C>, Option<D>)> = {
            let mut vec = Vec::new();
            vec.resize_with(builder.get_component_count(), || (None, None, None, None));
            vec
        };

        A::fill_iterator(builder, vec.iter_mut().map(|(a, _, _, _)| a))?;
        B::fill_iterator(builder, vec.iter_mut().map(|(_, b, _, _)| b))?;
        C::fill_iterator(builder, vec.iter_mut().map(|(_, _, c, _)| c))?;
        D::fill_iterator(builder, vec.iter_mut().map(|(_, _, _, d)| d))?;
        Ok(unsafe { unwrap_vec(vec) })
    }
}
