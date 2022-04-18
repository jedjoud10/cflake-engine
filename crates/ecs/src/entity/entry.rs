use super::Entity;
use crate::{registry, Archetype, Component, EcsManager, EntryError, Mask};

// An entity entry that we can use to access multiple components on a single entity
pub struct Entry<'a> {
    // Internal query for fetching components
    //query: EntityEntryQuery<'a>,
    archetype: &'a Archetype,
    bundle: usize,
}

impl<'a> Entry<'a> {
    // Create an entry from the Ecs manager and an entity
    pub(crate) fn new(manager: &'a mut EcsManager, entity: Entity) -> Option<Self> {
        let linkings = manager.entities.get(entity)?;
        Some(Self {
            archetype: manager.archetypes.get(&linkings.mask)?,
            bundle: linkings.bundle,
        })
    }
    // Try to get a component mask
    fn mask<T: Component>(&self) -> Result<Mask, EntryError> {
        // Get le mask
        let mask = registry::mask::<T>();

        // Handle unlinked components
        if self.archetype.mask & mask != mask {
            return Err(EntryError::MissingComponent(registry::name::<T>()));
        }

        Ok(mask)
    }
    // Get a pointer to a linked component
    pub unsafe fn get_ptr<T: Component>(&self) -> Result<*mut T, EntryError> {
        let mask = self.mask::<T>()?;
        let ptr = self.archetype.vectors[&mask].get_storage_ptr();
        Ok(ptr.cast::<T>().as_ptr().add(self.bundle))
    }
    // Get an immutable reference to a linked component
    pub fn get<T: Component>(&self) -> Result<&T, EntryError> {
        unsafe { self.get_ptr::<T>().map(|ptr| &*ptr) }
    }
    // Get a mutable reference to a linked component
    pub fn get_mut<T: Component>(&mut self) -> Result<&mut T, EntryError> {
        // Update the mutation state
        let mask = self.mask::<T>()?;
        self.archetype.states.update(self.bundle, |row| row.update(|_, m| m.set(mask.offset(), true)));

        self.get_mut()
    }
    // Get a mutable reference to a linked component silently, without triggering a mutation state change
    pub fn get_mut_silent<T: Component>(&mut self) -> Result<&mut T, EntryError> {
        unsafe { self.get_ptr::<T>().map(|ptr| &mut *ptr) }
    }
    // Check if a specific component was mutated
    pub fn was_mutated<T: Component>(&self) -> Result<bool, EntryError> {
        let mask = self.mask::<T>()?;
        let mutated = self.archetype.states.get(self.bundle).unwrap();
        Ok(mutated.mutated(mask.offset()))
    }
}
