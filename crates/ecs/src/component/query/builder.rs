use crate::{registry, Component, EcsManager, Mask, MutQuery, QueryError, RefQuery};
use std::cell::{RefCell, UnsafeCell};

// Helps us get queries from archetypes
pub struct QueryBuilder<'a> {
    // Ecs Manager
    pub(super) manager: &'a EcsManager,

    // Internal entry mask
    pub(super) mask: Mask,

    // The queries that are currently being mutably borrowed
    pub(super) borrowed: RefCell<Mask>,
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
    pub(super) fn get_component_mask<T: Component>(&self) -> Result<Mask, QueryError> {
        // Component mask
        let mask = registry::mask::<T>().map_err(QueryError::ComponentError)?;

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
    pub fn get<T: Component + RefQuery>(&self) -> Result<Vec<&T>, QueryError> {
        T::query(self)
    }
    // Create a new mutable query
    pub fn get_mut<T: Component + MutQuery>(&self) -> Result<Vec<&mut T>, QueryError> {
        T::query_mut(self)
    }
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
