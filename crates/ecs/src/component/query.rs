use parking_lot::{MappedRwLockReadGuard, RwLockReadGuard};
use std::{
    any::type_name,
    cell::{RefCell, UnsafeCell},
    collections::BTreeMap,
};
use tinyvec::ArrayVec;

use super::{registry, Component, ComponentState, QueryError};
use crate::{Archetype, EcsManager, Mask};

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
    // Get the component vec storage directly from an archetype
    fn get_component_vec<'b, T: Component>(archetype: &'b Archetype, m_component: Mask) -> MappedRwLockReadGuard<'b, Vec<UnsafeCell<T>>> {
        // A simple downcast ref
        RwLockReadGuard::map(archetype.components().read(), |read| {
            let (storage, _) = read.get(&m_component).unwrap();
            storage.as_any().downcast_ref::<Vec<UnsafeCell<T>>>().unwrap()
        })
    }
    // Create a new immutable query
    pub fn get<T: Component>(&self) -> Result<Vec<&T>, QueryError> {
        // Get the component mask and entry mask
        let component_mask = self.get_component_mask::<T>()?;
        let entry_mask = self.mask;

        // A vector full of references
        let mut components = Vec::<&T>::new();
        for (&archetype_mask, archetype) in self.manager.archetypes.iter() {
            // Check if the archetype is valid for this query builder
            if entry_mask & archetype_mask == entry_mask {
                // Extend the components by the components stored in this archetype (rip performance)
                let vec = Self::get_component_vec::<T>(archetype, component_mask);
                components.extend(vec.iter().map(|cell| unsafe { &*cell.get() }))
            }
        }

        Ok(components)
    }
    // Create a new mutable query
    pub fn get_mut<T: Component>(&self) -> Result<Vec<&mut T>, QueryError> {
        // Get the component mask and entry mask
        let component_mask = self.get_component_mask::<T>()?;
        let entry_mask = self.mask;

        // A vector full of mutable references
        let mut components = Vec::<&mut T>::new();

        for (&archetype_mask, archetype) in self.manager.archetypes.iter() {
            // Check if the archetype is valid for this query builder
            if entry_mask & archetype_mask == entry_mask {
                // Extend the components by the components stored in this archetype (rip performance)
                let vec = Self::get_component_vec::<T>(archetype, component_mask);
                components.extend(vec.iter().map(|cell| unsafe { &mut *cell.get() }))
            }
        }

        // The component is currently being borro
        let mut borrowed = self.borrowed.borrow_mut();
        *borrowed = *borrowed | component_mask;

        Ok(components)
    }
    // Get the states vector of a specific component
    pub fn states<T: Component>(&self) -> Result<Vec<ComponentState>, QueryError> {
        // Get the component mask and entry mask
        let component_mask = self.get_component_mask::<T>()?;
        let entry_mask = self.mask;

        // A vector full of states
        let mut components = Vec::<ComponentState>::new();

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

        Ok(components)
    }
    /*
    // Get the mutation states of each component of a specific type
    pub fn mutations<T: Component>(&self) -> Result<Vec<bool>, QueryError> {
        // Get the component mask and entry mask
        let component_mask = self.get_component_mask::<T>()?;
        let entry_mask = self.mask;

        // The mutation states
        let mut mutated = Vec::<bool>::new();

        for (&archetype_mask, archetype) in self.manager.archetypes.iter() {
            // Check if the archetype is valid for this query builder
            if entry_mask & archetype_mask == entry_mask {
                // Get the mutation states
                let vec = Self::get_mutation_vec::<T>(archetype, component_mask);
                components.extend(vec.iter().map(|cell| unsafe { &mut *cell.get() }))
            }
        }

        // The component is currently being borro
        let mut borrowed = self.borrowed.borrow_mut();
        *borrowed = *borrowed | component_mask;
    }
    */
    // Get a raw mutable pointer to a component from an archetype mask and bundle index
    pub fn get_ptr<T: Component>(&self, bundle: usize, m_archetype: Mask) -> Result<*mut T, QueryError> {
        // Get the component mask
        let m_component = self.get_component_mask::<T>()?;

        // And then get the singular component
        let archetype = self
            .manager
            .archetypes
            .get(&m_archetype)
            .ok_or_else(|| QueryError::DirectAccessArchetypeMissing(m_archetype, registry::name::<T>()))?;
        let vec = Self::get_component_vec::<T>(archetype, m_component);
        let component = vec.get(bundle).ok_or_else(|| QueryError::DirectAccessBundleIndexInvalid(bundle, registry::name::<T>()))?;
        Ok(component.get())
    }
}
