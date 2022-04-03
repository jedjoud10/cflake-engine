use crate::{registry, Component, Entity, Mask, QueryBuilder, QueryError};

// Something that can be queried. This will be implement on &T and &mut T (where T is Component)
pub trait QueryItem: Sized {
    // Get a vector that contains this queriable
    fn query<Layout: LayoutQuery, I: Iterator<Item = Self>>(builder: &QueryBuilder<Layout>) -> Result<I, QueryError>;
    // Try to get the mask of this query item. This might fail in the case of a QueryItem of type Entity
    fn try_get_mask() -> Option<Mask>;
}

// QueryItem implementations
impl<T: Component> QueryItem for &T {
    // Get some immutable references
    fn query<Layout: LayoutQuery, I: Iterator<Item = Self>>(builder: &QueryBuilder<Layout>) -> Result<I, QueryError> {
        let refs: I = builder.get_component_vec_mapped::<T, &T, _>(|cell| unsafe { &*cell.get() })?;
        Ok(refs)
    }

    // Get the component mask
    fn try_get_mask() -> Option<Mask> {
        registry::mask::<T>().ok()
    }
}
impl<T: Component> QueryItem for &mut T {
    // Get some mutable references
    fn query<Layout: LayoutQuery, I: Iterator<Item = Self>>(builder: &QueryBuilder<Layout>) -> Result<I, QueryError> {
        let mut_refs = builder.get_component_vec_mapped::<T, &mut T, _>(|cell| unsafe { &mut *cell.get() })?;
        //Ok(mut_refs)
        todo!()
    }

    // Get the component mask
    fn try_get_mask() -> Option<Mask> {
        registry::mask::<T>().ok()
    }
}
impl QueryItem for Entity {
    // Get some immutable references to the entity IDs
    fn query<Layout: LayoutQuery>(builder: &QueryBuilder<Layout>) -> Result<Vec<Self>, QueryError> {
        Ok(builder.filter_archetypes().flat_map(|archetype| archetype.entities()).cloned().collect::<Vec<_>>())
    }

    // Entity IDs can't have masks
    fn try_get_mask() -> Option<Mask> {
        None
    }
}

// Layout query that contains multiple QueryItems
pub trait LayoutQuery {
    type Layout;
    // Calculate the mask using the current layout
    fn mask() -> Mask;
}

// LayoutQuery implementations
impl<A: QueryItem> LayoutQuery for A {
    type Layout = A;

    fn mask() -> Mask {
        A::try_get_mask().unwrap_or_default()
    }
}
impl<A: QueryItem, B: QueryItem> LayoutQuery for (A, B) {
    type Layout = (A, B);

    fn mask() -> Mask {
        A::try_get_mask().unwrap_or_default() |
        B::try_get_mask().unwrap_or_default()
    }
}
impl<A: QueryItem, B: QueryItem, C: QueryItem> LayoutQuery for (A, B, C) {
    type Layout = (A, B, C);

    fn mask() -> Mask {
        A::try_get_mask().unwrap_or_default() |
        B::try_get_mask().unwrap_or_default() |
        C::try_get_mask().unwrap_or_default()
    }
}
impl<A: QueryItem, B: QueryItem, C: QueryItem, D: QueryItem> LayoutQuery for (A, B, C, D) {
    type Layout = (A, B, C, D);

    fn mask() -> Mask {
        A::try_get_mask().unwrap_or_default() |
        B::try_get_mask().unwrap_or_default() |
        C::try_get_mask().unwrap_or_default() |
        D::try_get_mask().unwrap_or_default()
    }
}
