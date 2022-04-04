use smallvec::SmallVec;

use crate::{registry, Archetype, Component, EcsManager, Entity, LayoutQuery, Mask, QueryError, ARCHETYPE_INLINE_SIZE, ComponentError, EntityLinkings};
use std::{
    cell::{RefCell, UnsafeCell, Ref, RefMut},
    marker::PhantomData,
};

// Helper functions for Query and EntryQuery
// Get a specific component mask using our current query mask (faillable)
// This function cannot be called two or more times with the same component type
fn steal_component_mask<T: Component>(mask: Mask, accessed: RefMut<Mask>) -> Result<Mask, QueryError> {
    // Component mask
    let mask = registry::mask::<T>().map_err(QueryError::ComponentError)?;

    // Check if the component mask is even valid
    if mask & mask == Mask::default() {
        return Err(QueryError::NotLinked(registry::name::<T>()));
    }

    // Check if the component is currently mutably borrowed
    if mask & *accessed != Mask::default() {
        return Err(QueryError::AlreadyBorrowed(registry::name::<T>()));
    }

    // The component was borrowed, we cannot access it again
    *accessed = *accessed | mask;

    Ok(mask)
}

// Helps us get queries from archetypes
pub struct Query<'a, Layout: LayoutQuery> {
    // Ecs Manager
    pub(super) manager: &'a EcsManager,

    // Internal entry mask (calculated from the Layout)
    pub(super) mask: Mask,

    // The queries that are currently being mutably borrowed
    pub(super) accessed: RefCell<Mask>,
    _phantom: PhantomData<Layout>,
}

impl<'a, Layout: LayoutQuery> Query<'a, Layout> {
    // Create a new query builder from a layout
    pub fn new(manager: &'a mut EcsManager) -> Result<Self, QueryError> {
        Ok(Self {
            manager,
            mask: Layout::mask().map_err(QueryError::ComponentError)?,
            accessed: Default::default(),
            _phantom: Default::default(),
        })
    }
    // Get the query components in their respective layout
    pub fn fetch(self) -> Result<Vec<Layout>, QueryError> {
        Layout::query(&self)
    }

    /* #region Helper functions */

    // Filter the archetypes based on the interally stored mask
    fn filter_archetypes(&self) -> impl Iterator<Item = &Archetype> {
        self.manager.archetypes.iter().filter(move |archetype| self.mask & archetype.mask() == self.mask)
    }
    // Count the number of entities
    pub fn count(&self) -> usize {
        self.filter_archetypes().map(|archetype| archetype.entities().len()).sum::<usize>()
    }
    // Get a vector that contains all the underlying components
    pub(crate) fn get_cells<T: Component>(&self) -> Result<impl Iterator<Item = &UnsafeCell<T>>, QueryError> {
        let mask = steal_component_mask::<T>(self.mask, self.accessed.borrow_mut())?;

        Ok(self.filter_archetypes().flat_map(move |archetype| {
            // Fetch the components
            let vec = archetype.vectors().get(&mask).unwrap();
            let vec = vec.as_any().downcast_ref::<Vec<UnsafeCell<T>>>().unwrap();
            vec.iter()
        }))
    }
    /* #endregion */
}

// Query for use inside an entry
pub struct EntityEntryQuery<'a> {
    // Ecs Manager
    pub(super) manager: &'a EcsManager,

    // The entity bundle index
    pub(super) bundle: usize,

    // The archetype where our entity components are stored
    pub(super) archetype: &'a Archetype,

    // The queries that are currently being mutably borrowed
    pub(super) accessed: RefCell<Mask>,
}

impl<'a> EntityEntryQuery<'a> {
    // Create a new query from a specific entity
    pub fn new(manager: &'a mut EcsManager, entity: Entity) -> Option<Self> {
        // Get the entity linkings
        let linkings = manager.entities.get(entity).and_then(|x| x.as_ref())?;
        if !linkings.is_valid() { return None }

        // And then get the singular component
        let archetype = manager.archetypes.get(&linkings.mask).unwrap();

        Some(Self {
            manager,
            archetype,
            bundle: linkings.bundle,
            accessed: Default::default(),
        })
    }
    // Get a pointer to a component that is linked to our entity
    fn get_ptr<T: Component>(&self) -> Result<*mut T, QueryError> {
        let component_mask = steal_component_mask::<T>(self.linkings.mask, self.accessed.borrow_mut())?;
        let storage = self.archetype.vectors().get(&component_mask).unwrap();
        let vec = storage.as_any().downcast_ref::<Vec<UnsafeCell<T>>>().unwrap();
        let component = vec.get(self.bundle).unwrap();
        
        
        


        Ok(component.get())
    }
    // Get (immutable) and get mut (mutable)
    pub(super) fn get<T: Component>() -> Result<&T, QueryError> {

    }
    pub(super) fn get<T: Component>() -> Result<&T, QueryError> {
        
    }
}