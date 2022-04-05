use crate::{registry, Archetype, Component, EcsManager, Entity, LayoutQuery, Mask, QueryError};
use std::{cell::UnsafeCell, marker::PhantomData};

// Helper functions for Query and EntryQuery
// Get a specific component mask using our current query mask (faillable)
// This function cannot be called two or more times with the same component type
fn get_component_mask<T: Component>(entry: Mask) -> Result<Mask, QueryError> {
    // Component mask
    let mask = registry::mask::<T>().map_err(QueryError::ComponentError)?;

    // Check if the component mask is even valid
    if entry & mask == Mask::default() {
        return Err(QueryError::NotLinked(registry::name::<T>()));
    }

    Ok(mask)
}

// Helps us get queries from archetypes
pub struct Query;

impl<'a> Query {
    // Get the filtered archetypes
    fn filtered(manager: &'a EcsManager, mask: Mask) -> impl Iterator<Item = &'a Archetype> {
        manager
            .archetypes
            .iter()
            .filter(move |archetype| mask & archetype.mask() == mask)
    }
    // Create a new query builder from a layout
    pub fn new<Layout: LayoutQuery<'a>>(manager: &'a mut EcsManager) -> Result<Vec<Layout>, QueryError> {
        // Get layout mask since we must do validity checks on each archetype
        let mask = Layout::mask().map_err(QueryError::ComponentError)?;

        // Entity count first
        let count = Self::filtered(manager, mask)
            .map(|archetype| archetype.entities().len())
            .sum::<usize>();
        
        // Get the iterator for the layout
        let iter = Self::filtered(manager, mask);

        Layout::query_from_archetypes(iter, count)
    }
}

// Query for use inside an entry
pub(crate) struct EntityEntryQuery<'a> {
    bundle: usize,
    archetype: &'a Archetype,
}

impl<'a> EntityEntryQuery<'a> {
    // Create a new query from a specific entity
    pub(crate) fn new(manager: &'a mut EcsManager, entity: Entity) -> Option<Self> {
        // Get the entity linkings
        let linkings = manager.entities.get(entity)?;

        // And then get the singular component
        let archetype = manager.archetypes.get(&linkings.mask).unwrap();

        Some(Self {
            archetype,
            bundle: linkings.bundle,
        })
    }
    // Get a pointer to a component that is linked to our entity
    fn get_ptr<T: Component>(&self) -> Result<*mut T, QueryError> {
        let component_mask = get_component_mask::<T>(self.archetype.mask())?;
        let storage = self.archetype.vectors().get(&component_mask).unwrap();
        let vec = storage.as_any().downcast_ref::<Vec<UnsafeCell<T>>>().unwrap();
        let component = vec.get(self.bundle).unwrap();
        Ok(component.get())
    }
    // Get (immutable) and get mut (mutable)
    pub(crate) fn get<T: Component>(&self) -> Result<&T, QueryError> {
        self.get_ptr().map(|ptr| unsafe { &*ptr })
    }
    pub(crate) fn get_mut<T: Component>(&mut self) -> Result<&mut T, QueryError> {
        self.get_ptr().map(|ptr| unsafe { &mut *ptr })
    }
}
