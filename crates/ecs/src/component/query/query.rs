use std::cell::UnsafeCell;
use crate::{Component, Entity, QueryBuilder, QueryError, EntityState};

// TODO: Fix duplicate code

// Something that can be queried using the query builder
pub trait RefQuery<'a> {
    // The element of the result vector
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

        // The component is currently being borrowed
        let mut borrowed = builder.borrowed.borrow_mut();
        *borrowed = *borrowed | component_mask;

        Ok(components)
    }
}
impl<'a, T: Component> RefQuery<'a> for (T, bool) {
    // We can also fetch the component mutation state of each component
    type Item = bool;
    fn query(builder: &QueryBuilder<'a>) -> Result<Vec<Self::Item>, QueryError> {
        // Get the component mask and entry mask
        let component_mask = builder.get_component_mask::<T>()?;
        let entry_mask = builder.mask;

        // A vector full of references to component mutation states
        let mut mutated = Vec::<bool>::new();
        for archetype in builder.manager.archetypes.iter() {
            // Check if the archetype is valid for our query builder
            if entry_mask & archetype.mask() == entry_mask {
                // Fetch the vector
                let (_storage, storage_states) = archetype.components().get(&component_mask).unwrap();

                // Extend
                mutated.extend(storage_states.iter())
            }
        }
        Ok(mutated)
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

impl<'a> RefQuery<'a> for EntityState {
    // We can also fetch the entity states
    type Item = EntityState;
    fn query(builder: &QueryBuilder<'a>) -> Result<Vec<Self::Item>, QueryError> {
        // Just get the entry mask this time
        let entry_mask = builder.mask;

        // A vector full of entity states
        let mut entities = Vec::<EntityState>::new();
        for archetype in builder.manager.archetypes.iter() {
            // Check if the archetype is valid for our query builder
            if entry_mask & archetype.mask() == entry_mask {
                // Extend
                entities.extend(archetype.states().iter())
            }
        }
        Ok(entities)
    }
}

// Something that can be mutably queried using the query builder
pub trait MutQuery<'a> {
    // The element of the result vector
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

        // The component is currently being borrowed
        let mut borrowed = builder.borrowed.borrow_mut();
        *borrowed = *borrowed | component_mask;

        Ok(components)
    }
}
