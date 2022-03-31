use parking_lot::{MappedRwLockReadGuard, RwLockReadGuard};
use std::{
    any::type_name,
    cell::{RefCell, UnsafeCell},
    collections::BTreeMap,
};
use tinyvec::ArrayVec;

use super::{registry, Component, ComponentState, QueryError};
use crate::{Archetype, EcsManager, Mask};

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


// Helps us get queries from archetypes
pub struct QueryBuilder<'a> {
    // Ecs Manager
    manager: &'a EcsManager,

    // Internal entry mask
    mask: Mask,

    // The queries that are currently being mutably borrowed
    borrowed: RefCell<Mask>,
}

impl<'a> QueryBuilder<'a> {
    // Create self from the Ecs manager and some masks
    pub fn new(manager: &'a mut EcsManager, mask: Mask) -> Self {
        Self {
            manager,
            mask,
            borrowed: Default::default(),
        }
    }
    // This will get the component mask, not the entry mask
    fn get_component_mask<T: Component>(&self) -> Result<Mask, QueryError> {
        // Component mask
        let mask = registry::mask::<T>().map_err(|err| QueryError::ComponentError(err))?;

        // Check if the component mask is even valid
        if mask & self.mask == Mask::default() {
            return Err(QueryError::Unlinked(registry::name::<T>()));
        }

        // Check if the component is currently mutably borrowed
        if mask & *self.borrowed.borrow() != Mask::default() {
            return Err(QueryError::MutablyBorrowed(registry::name::<T>()));
        }

        Ok(mask)
    }
    // Create a new immutable query
    pub fn get<T: Component + RefQuery>(&self) -> Result<Vec<&T>, QueryError> { T::query(self) }
    // Create a new mutable query
    pub fn get_mut<T: Component + MutQuery>(&self) -> Result<Vec<&mut T>, QueryError> { T::query_mut(self) }
    // Get a raw mutable pointer to a component from an archetype mask and bundle index
    pub fn get_ptr<T: Component>(&self, bundle: usize, m_archetype: Mask) -> Result<*mut T, QueryError> {
        // Get the component mask
        let component_mask = self.get_component_mask::<T>()?;

        // And then get the singular component
        let archetype = self
            .manager
            .archetypes
            .get(&m_archetype)
            .ok_or_else(|| QueryError::DirectAccessArchetypeMissing(m_archetype, registry::name::<T>()))?;

        // Read from the rwlock
        let (storage, _) = archetype.components().get(&component_mask).unwrap();

        // Just fetch the pointer
        let vec = storage.as_any().downcast_ref::<Vec<UnsafeCell<T>>>().unwrap();
        let component = vec.get(bundle).ok_or_else(|| QueryError::DirectAccessBundleIndexInvalid(bundle, registry::name::<T>()))?;
        Ok(component.get())
    }
}

/*

for (&archetype_mask, archetype) in self.manager.archetypes.iter() {
            // Check if the archetype is valid for this query builder
            if entry_mask & archetype_mask == entry_mask {
                // Get the component states for this specific archetype, and then add it to the vector
                let read = archetype.components().read();
                let (_, states) = read.get(&component_mask).unwrap();
                let len = archetype.entities().len();
                for bundle in 0..len {
                    components.push(states.get(bundle));
                }
            }
        }
*/
