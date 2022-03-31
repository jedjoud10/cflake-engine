use parking_lot::{MappedRwLockReadGuard, RwLockReadGuard};
use std::{
    any::type_name,
    cell::{RefCell, UnsafeCell},
    collections::BTreeMap,
};
use tinyvec::ArrayVec;

use crate::{Archetype, Component, ComponentState, EcsManager, Entity, Mask, QueryBuilder, QueryError};

// Something that can be queried using the query builder. This will return a vector of type Vec<&Self>
pub trait RefQuery {
    // Create a vector (full of immutable references) using a query builder
    fn query<'a>(builder: &QueryBuilder<'a>) -> Result<Vec<&'a Self>, QueryError>;
}

impl<T: Component> RefQuery for T {
    // Components are always queriable
    fn query<'a>(builder: &QueryBuilder<'a>) -> Result<Vec<&'a Self>, QueryError> {
        // Get the component mask and entry mask
        let component_mask = builder.get_component_mask::<T>()?;
        let entry_mask = builder.mask;

        // A vector full of references
        let mut components = Vec::<&T>::new();
        for (&archetype_mask, archetype) in builder.manager.archetypes.iter() {
            // Check if the archetype is valid for our query builder
            if entry_mask & archetype_mask == entry_mask {
                // Fetch the vector
                let (storage, _) = archetype.components().get(&component_mask).unwrap();

                // Extend
                let vec = storage.as_any().downcast_ref::<Vec<UnsafeCell<T>>>().unwrap();
                components.extend(vec.iter().map(|cell| unsafe { &*cell.get() }))
            }
        }
        Ok(components)
    }
}

impl RefQuery for ComponentState {
    // We can also fetch the component state of each component
    fn query<'a>(builder: &QueryBuilder<'a>) -> Result<Vec<&'a Self>, QueryError> {
        todo!()
    }
}

impl RefQuery for Entity {
    // We can also fetch the entity that is linked to each component bundle
    fn query<'a>(builder: &QueryBuilder<'a>) -> Result<Vec<&'a Self>, QueryError> {
        todo!()
    }
}

// Something that can be mutably queried using the query builder. This will return a vector of type Vec<&mut Self>
pub trait MutQuery {
    // Create a vector (full of mutable references) using a query builder
    fn query_mut<'a>(builder: &QueryBuilder<'a>) -> Result<Vec<&'a mut Self>, QueryError>;
}

impl<T: Component> MutQuery for T {
    // Components are always queriable
    fn query_mut<'a>(builder: &QueryBuilder<'a>) -> Result<Vec<&'a mut Self>, QueryError> {
        // Get the component mask and entry mask
        let component_mask = builder.get_component_mask::<T>()?;
        let entry_mask = builder.mask;

        // A vector full of mtuable references
        let mut components = Vec::<&mut T>::new();
        for (&archetype_mask, archetype) in builder.manager.archetypes.iter() {
            // Check if the archetype is valid for our query builder
            if entry_mask & archetype_mask == entry_mask {
                // Fetch the vector and states
                let (storage, states) = archetype.components().get(&component_mask).unwrap();

                // Set all the states to mutated, since we will be reading from this query mutably
                states.set_all_mutated();

                // Extend
                let vec = storage.as_any().downcast_ref::<Vec<UnsafeCell<T>>>().unwrap();
                components.extend(vec.iter().map(|cell| unsafe { &mut *cell.get() }))
            }
        }

        // The component is currently being borro
        let mut borrowed = builder.borrowed.borrow_mut();
        *borrowed = *borrowed | component_mask;

        Ok(components)
    }
}
