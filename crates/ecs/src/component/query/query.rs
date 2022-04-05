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
pub struct Query<'a, Layout: LayoutQuery<'a>> {
    manager: &'a EcsManager,
    mask: Mask,
    _phantom: PhantomData<Layout>,
}

impl<'a, Layout: LayoutQuery<'a> + 'a> Query<'a, Layout> {
    // Create a new query builder from a layout
    pub fn new(manager: &'a mut EcsManager) -> Result<Self, QueryError> {
        Ok(Self {
            manager,
            mask: Layout::mask().map_err(QueryError::ComponentError)?,
            _phantom: Default::default(),
        })
    }
    // Get the query components in their respective layout
    pub fn fetch(self) -> Result<Vec<Layout>, QueryError> {
        Layout::query_from_archetypes(self.get_filtered_archetypes(), self.count())
    }
    // Get the filtered archetypes
    fn get_filtered_archetypes(&self) -> impl Iterator<Item = &'a Archetype> + '_ {
        self.manager.archetypes.iter().filter(|archetype| self.mask & archetype.mask() == self.mask)
    }
    // Count the number of entities that are valid for this query
    fn count(&self) -> usize {
        self.get_filtered_archetypes().map(|archetype| archetype.entities().len()).sum::<usize>()
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
