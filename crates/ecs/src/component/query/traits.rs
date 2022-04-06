use crate::{registry, Archetype, Component, ComponentError, ComponentFlagLanesIter, Entity, EntityState, EntityStatesIter, FlagLane, Mask, QueryError};
use itertools::izip;
use std::cell::UnsafeCell;

// This hurts my eyes
// Pls don't touch dis. -Jed 11:49pm 04/04/2022

// Something that can be queried. This will be implement on &T and &mut T (where T is Component). This will also be implemented on &Entity and &BundleData
pub trait QueryItem<'a> {
    // Iterator shit
    type Iter: Iterator<Item = Self::Item>;
    type Item: Send;

    // Get a custom iterator from an archetype
    // This will later be zipped with other iterators to form a query
    fn archetype_map_iter(archetype: &'a Archetype) -> Self::Iter;
    fn try_get_mask() -> Result<Mask, ComponentError> {
        Ok(Mask::default())
    }
}

// TODO: Cache "let mask = registry::mask::<T>().unwrap();" somehow

// QueryItem implementations
impl<'a, T: Component> QueryItem<'a> for &'a T {
    type Iter = std::iter::Map<std::slice::Iter<'a, UnsafeCell<T>>, fn(&UnsafeCell<T>) -> Self>;
    type Item = Self;
    fn archetype_map_iter(archetype: &'a Archetype) -> Self::Iter {
        let mask = registry::mask::<T>().unwrap();
        let vec = archetype.vectors().get(&mask).unwrap();
        let vec = vec.as_any().downcast_ref::<Vec<UnsafeCell<T>>>().unwrap();
        vec.iter().map(|cell| unsafe { &*cell.get() })
    }
    fn try_get_mask() -> Result<Mask, ComponentError> {
        registry::mask::<T>()
    }
}
impl<'a, T: Component> QueryItem<'a> for &'a mut T {
    type Iter = std::iter::Map<std::slice::Iter<'a, UnsafeCell<T>>, fn(&UnsafeCell<T>) -> Self>;
    type Item = Self;
    fn archetype_map_iter(archetype: &'a Archetype) -> Self::Iter {
        let mask = registry::mask::<T>().unwrap();
        let vec = archetype.vectors().get(&mask).unwrap();
        let vec = vec.as_any().downcast_ref::<Vec<UnsafeCell<T>>>().unwrap();
        //archetype.states().set_all_component_states(mask, true).unwrap();

        vec.iter().map(|cell| unsafe { &mut *cell.get() })
    }
    fn try_get_mask() -> Result<Mask, ComponentError> {
        registry::mask::<T>()
    }
}
impl<'a> QueryItem<'a> for &'a Entity {
    type Iter = std::slice::Iter<'a, Entity>;
    type Item = Self;
    fn archetype_map_iter(archetype: &'a Archetype) -> Self::Iter {
        archetype.entities().iter()
    }
}
impl<'a> QueryItem<'a> for &'a EntityState {
    type Iter = EntityStatesIter<'a>;
    type Item = EntityState;
    fn archetype_map_iter(archetype: &'a Archetype) -> Self::Iter {
        archetype.states().iter_entity_states()
    }
}
impl<'a> QueryItem<'a> for &'a FlagLane {
    type Iter = ComponentFlagLanesIter<'a>;
    type Item = FlagLane;
    fn archetype_map_iter(archetype: &'a Archetype) -> Self::Iter {
        archetype.states().iter_component_states_lanes()
    }
}

// Layout query that contains multiple QueryItems
pub trait LayoutQuery<'a> {
    type Item: Send;
    fn mask() -> Result<Mask, ComponentError>;
    fn query_from_archetypes(archetypes: impl Iterator<Item = &'a Archetype>, count: usize) -> Result<Vec<Self::Item>, QueryError>;
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
    type Item = A::Item;

    fn mask() -> Result<Mask, ComponentError> {
        A::try_get_mask()
    }

    fn query_from_archetypes(archetypes: impl Iterator<Item = &'a Archetype>, count: usize) -> Result<Vec<Self::Item>, QueryError> {
        Ok(into_vec(count, archetypes.flat_map(|archetype| A::archetype_map_iter(archetype))))
    }
}
impl<'a, A: QueryItem<'a>, B: QueryItem<'a>> LayoutQuery<'a> for (A, B) {
    type Item = (A::Item, B::Item);

    fn mask() -> Result<Mask, ComponentError> {
        Ok(A::try_get_mask()? | B::try_get_mask()?)
    }

    fn query_from_archetypes(archetypes: impl Iterator<Item = &'a Archetype>, count: usize) -> Result<Vec<Self::Item>, QueryError> {
        Ok(into_vec(
            count,
            archetypes.flat_map(|archetype| izip!(A::archetype_map_iter(archetype), B::archetype_map_iter(archetype))),
        ))
    }
}
impl<'a, A: QueryItem<'a>, B: QueryItem<'a>, C: QueryItem<'a>> LayoutQuery<'a> for (A, B, C) {
    type Item = (A::Item, B::Item, C::Item);

    fn mask() -> Result<Mask, ComponentError> {
        Ok(A::try_get_mask()? | B::try_get_mask()? | C::try_get_mask()?)
    }

    fn query_from_archetypes(archetypes: impl Iterator<Item = &'a Archetype>, count: usize) -> Result<Vec<Self::Item>, QueryError> {
        Ok(into_vec(
            count,
            archetypes.flat_map(|archetype| izip!(A::archetype_map_iter(archetype), B::archetype_map_iter(archetype), C::archetype_map_iter(archetype))),
        ))
    }
}
impl<'a, A: QueryItem<'a>, B: QueryItem<'a>, C: QueryItem<'a>, D: QueryItem<'a>> LayoutQuery<'a> for (A, B, C, D) {
    type Item = (A::Item, B::Item, C::Item, D::Item);

    fn mask() -> Result<Mask, ComponentError> {
        Ok(A::try_get_mask()? | B::try_get_mask()? | C::try_get_mask()? | D::try_get_mask()?)
    }

    fn query_from_archetypes(archetypes: impl Iterator<Item = &'a Archetype>, count: usize) -> Result<Vec<Self::Item>, QueryError> {
        Ok(into_vec(
            count,
            archetypes.flat_map(|archetype| {
                izip!(
                    A::archetype_map_iter(archetype),
                    B::archetype_map_iter(archetype),
                    C::archetype_map_iter(archetype),
                    D::archetype_map_iter(archetype)
                )
            }),
        ))
    }
}
impl<'a, A: QueryItem<'a>, B: QueryItem<'a>, C: QueryItem<'a>, D: QueryItem<'a>, E: QueryItem<'a>> LayoutQuery<'a> for (A, B, C, D, E) {
    type Item = (A::Item, B::Item, C::Item, D::Item, E::Item);

    fn mask() -> Result<Mask, ComponentError> {
        Ok(A::try_get_mask()? | B::try_get_mask()? | C::try_get_mask()? | D::try_get_mask()? | E::try_get_mask()?)
    }

    fn query_from_archetypes(archetypes: impl Iterator<Item = &'a Archetype>, count: usize) -> Result<Vec<Self::Item>, QueryError> {
        Ok(into_vec(
            count,
            archetypes.flat_map(|archetype| {
                izip!(
                    A::archetype_map_iter(archetype),
                    B::archetype_map_iter(archetype),
                    C::archetype_map_iter(archetype),
                    D::archetype_map_iter(archetype),
                    E::archetype_map_iter(archetype)
                )
            }),
        ))
    }
}
