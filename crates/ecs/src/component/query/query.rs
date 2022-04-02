use std::cell::UnsafeCell;
use crate::{Component, Entity, QueryBuilder, QueryError, EntityState, ArchetypeBundle};

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
                let vec = archetype.vectors().get(&component_mask).unwrap();

                // Extend
                let vec = vec.as_any().downcast_ref::<Vec<UnsafeCell<T>>>().unwrap();
                components.extend(vec.iter().map(|cell| unsafe { &*cell.get() }))
            }
        }

        // The component was borrowed, we cannot access it again
        let mut accessed = builder.accessed.borrow_mut();
        *accessed = *accessed | component_mask;

        Ok(components)
    }
}

impl<'a> RefQuery<'a> for ArchetypeBundle {
    type Item = ArchetypeBundle;
    fn query(builder: &QueryBuilder<'a>) -> Result<Vec<Self::Item>, QueryError> {
        // Just get the entry mask this time
        let entry_mask = builder.mask;

        // A vector full of archetype bundles
        let mut bundles = Vec::<ArchetypeBundle>::new();
        for archetype in builder.manager.archetypes.iter() {
            // Check if the archetype is valid for our query builder
            if entry_mask & archetype.mask() == entry_mask {
                // Extend
                bundles.extend(archetype.entities().iter().enumerate().map(|(i, entity)| ArchetypeBundle::new(i, *entity, archetype)))
            }
        }
        Ok(bundles)
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
                let vec = &archetype.vectors()[&component_mask];

                // Set all the states to mutated, since we will be reading from this query mutably
                archetype.states().components[&component_mask].reset_to(true);

                // Extend
                let vec = vec.as_any().downcast_ref::<Vec<UnsafeCell<T>>>().unwrap();
                components.extend(vec.iter().map(|cell| unsafe { &mut *cell.get() }))
            }
        }

        // The component was borrowed, we cannot access it again
        let mut accessed = builder.accessed.borrow_mut();
        *accessed = *accessed | component_mask;

        Ok(components)
    }
}
