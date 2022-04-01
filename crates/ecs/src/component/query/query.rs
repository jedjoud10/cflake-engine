use std::cell::UnsafeCell;

use crate::{Component, ComponentState, Entity, QueryBuilder, QueryError};

// Something that can be queried using the query builder. This will return a vector of type Vec<&Self>
pub trait RefQuery<'a> {
    type Item;
    // Create a vector (full of immutable references) using a query builder
    fn query(builder: &QueryBuilder<'a>) -> Result<Vec<Self::Item>, QueryError>;
}

impl<'a, T: Component> RefQuery<'a> for T {
    // Components are always queriable
    type Item = &'a T;
    fn query(builder: &QueryBuilder<'a>) -> Result<Vec<Self::Item>, QueryError> {
        // Get the component mask and entry mask
        let component_mask = builder.get_component_mask::<T>()?;
        let entry_mask = builder.mask;

        // A vector full of references
        let mut components = Vec::<&T>::new();
        for archetype in builder.manager.archetypes.iter() {
            // Check if the archetype is valid for our query builder
            if entry_mask & archetype.mask() == entry_mask {
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
impl<'a, T: Component> RefQuery<'a> for (T, ComponentState) {
    // We can also fetch the component state of each component
    type Item = ComponentState;
    fn query(builder: &QueryBuilder<'a>) -> Result<Vec<Self::Item>, QueryError> {
        // Get the component mask and entry mask
        let component_mask = builder.get_component_mask::<T>()?;
        let entry_mask = builder.mask;

        // A vector full of references to component states
        let mut states = Vec::<ComponentState>::new();
        for archetype in builder.manager.archetypes.iter() {
            // Check if the archetype is valid for our query builder
            if entry_mask & archetype.mask() == entry_mask {
                // Fetch the vector
                let (_storage, storage_states) = archetype.components().get(&component_mask).unwrap();

                // Extend
                states.extend(storage_states.iter())
            }
        }
        Ok(states)
    }
}

impl<'a> RefQuery<'a> for Entity {
    // We can also fetch the entity that is linked to each component bundle
    type Item = Entity;
    fn query(builder: &QueryBuilder<'a>) -> Result<Vec<Self::Item>, QueryError> {
        // Just get the entry mask this time
        let entry_mask = builder.mask;

        // A vector full of entity handles
        let mut entities = Vec::<Entity>::new();
        for archetype in builder.manager.archetypes.iter() {
            // Check if the archetype is valid for our query builder
            if entry_mask & archetype.mask() == entry_mask {
                // Extend
                entities.extend(archetype.entities())
            }
        }
        Ok(entities)
    }
}

// Something that can be mutably queried using the query builder. This will return a vector of type Vec<&mut Self>
pub trait MutQuery<'a> {
    type Item;
    // Create a vector (full of mutable references) using a query builder
    fn query_mut(builder: &QueryBuilder<'a>) -> Result<Vec<Self::Item>, QueryError>;
}

impl<'a, T: Component> MutQuery<'a> for T {
    // Components are always queriable
    type Item = &'a mut T;
    fn query_mut(builder: &QueryBuilder<'a>) -> Result<Vec<Self::Item>, QueryError> {
        // Get the component mask and entry mask
        let component_mask = builder.get_component_mask::<T>()?;
        let entry_mask = builder.mask;

        // A vector full of mtuable references
        let mut components = Vec::<&mut T>::new();
        for archetype in builder.manager.archetypes.iter() {
            // Check if the archetype is valid for our query builder
            if entry_mask & archetype.mask() == entry_mask {
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
