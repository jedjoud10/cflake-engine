use rayon::iter::IntoParallelIterator;

use crate::{registry, Archetype, Component, EcsManager, Entity, LayoutQuery, Mask, ParQueryIterator, QueryError, QueryIterator, EntityState};
use std::{cell::UnsafeCell, marker::PhantomData};

// A more efficient method to iterate through all the components in the world
pub struct Query;

impl<'a> Query {
    // Get the filtered archetypes
    fn filtered(manager: &'a EcsManager, mask: Mask) -> impl Iterator<Item = &'a Archetype> {
        manager.archetypes.iter().filter(move |archetype| mask & archetype.mask() == mask)
    }
    // Get a Vec<Layout> from the manager. This is it's own internal function because it will be used by new and par_new
    fn internal<Layout: LayoutQuery<'a>>(manager: &'a mut EcsManager) -> Result<Vec<Layout::Item>, QueryError> {
        // Get layout mask since we must do validity checks on each archetype
        let mask = Layout::mask().map_err(QueryError::ComponentError)?;

        // Entity count first
        let count = Self::filtered(manager, mask).map(|archetype| archetype.entities().len()).sum::<usize>();

        // Get the iterator for the layout
        let iter = Self::filtered(manager, mask);
        Layout::query_from_archetypes(iter, count)
    }

    // Create a new singlethreaded query from a layout
    pub fn new<Layout: LayoutQuery<'a>>(manager: &'a mut EcsManager) -> Result<QueryIterator<'a, Layout>, QueryError> {
        Ok(QueryIterator {
            iterator: Self::internal::<Layout>(manager)?.into_iter(),
            _phantom: PhantomData::default(),
        })
    }
    // Create a new multithreaded (using rayon) query from a layout
    pub fn par_new<Layout: LayoutQuery<'a>>(manager: &'a mut EcsManager) -> Result<ParQueryIterator<'a, Layout>, QueryError> {
        Ok(ParQueryIterator {
            iterator: Self::internal::<Layout>(manager)?.into_par_iter(),
            _phantom: PhantomData::default(),
        })
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
    // Get a specific component mask using our current query mask (faillable)
    fn get_component_mask<T: Component>(&self) -> Result<Mask, QueryError> {
        let mask = registry::mask::<T>().map_err(QueryError::ComponentError)?;
        if self.archetype.mask() & mask == Mask::default() {
            return Err(QueryError::NotLinked(registry::name::<T>()));
        }
        Ok(mask)
    }
    // Get a pointer to a component that is linked to our entity
    fn get_ptr<T: Component>(&self) -> Result<*mut T, QueryError> {
        let mask = self.get_component_mask::<T>()?;
        let component_mask = mask;
        let storage = self.archetype.vectors().get(&component_mask).unwrap();
        let vec = storage.as_any().downcast_ref::<Vec<UnsafeCell<T>>>().unwrap();
        let component = vec.get(self.bundle).unwrap();
        Ok(component.get())
    }

    // Get (immutable) and get mut (mutable)
    pub fn get<T: Component>(&self) -> Result<&T, QueryError> {
        self.get_ptr().map(|ptr| unsafe { &*ptr })
    }
    pub fn get_mut<T: Component>(&mut self) -> Result<&mut T, QueryError> {
        let mask = self.get_component_mask::<T>()?;
        self.archetype.states().set_component_state(self.bundle, mask, true);
        self.get_ptr().map(|ptr| unsafe { &mut *ptr })
    }

    // State getter
    pub fn was_mutated<T: Component>(&self) -> Result<bool, QueryError> {
        let mask = self.get_component_mask::<T>()?;
        Ok(self.archetype.states().get_component_state(self.bundle, mask).unwrap())
    }
    pub fn entity_state(&self) -> EntityState {
        self.archetype.states().get_entity_state(self.bundle).unwrap()
    }
}
