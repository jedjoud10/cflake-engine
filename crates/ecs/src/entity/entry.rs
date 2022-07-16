use std::marker::PhantomData;

use super::Entity;
use crate::{registry, Archetype, Component, EcsManager, EntryError, Mask, QueryLayout};

// An entity entry that we can use to access multiple components on a single entity
// This will take an immutable reference of the ECS manager, so we cannot mutate any of the underlying entity components
pub struct Entry<'a> {
    archetype: &'a Archetype,
    bundle: usize,
    _phantom: PhantomData<&'a EcsManager>,
}

impl<'a> Entry<'a> {
    /*
    // Create an entry from the Ecs manager and an entity
    pub(crate) fn new(manager: &'a EcsManager, entity: Entity) -> Option<Self> {
        let linkings = manager.entities.get(entity)?;
        Some(Self {
            archetype: manager.archetypes.get(&linkings.mask)?,
            bundle: linkings.bundle,
            _phantom: Default::default(),
        })
    }

    // Try to get a component mask
    fn mask<T: Component>(&self) -> Result<Mask, EntryError> {
        let mask = registry::mask::<T>();
        if self.archetype.mask() & mask != mask {
            return Err(EntryError::MissingComponent(registry::name::<T>()));
        }
        Ok(mask)
    }

    // Get an immutable reference to a linked component
    pub fn get<T: Component>(&self) -> Result<&T, EntryError> {
        let mask = self.mask::<T>()?;
        let boxed = self.archetype.storage().get(&mask).unwrap();
        let vec = boxed.as_any().downcast_ref::<Vec<T>>().unwrap();
        Ok(&vec[self.bundle])
    }

    // Check if a specific component was mutated
    pub fn was_mutated<T: Component>(&self) -> Result<bool, EntryError> {
        let mask = self.mask::<T>()?;
        let mutated = self.archetype.states().get(self.bundle).unwrap();
        Ok(mutated.mutated(mask.offset()))
    }
    */
}

// An entity entry that we can use to access multiple components on a single entity
// This will take a mutable reference of the ECS manager. Use Entry instead if you wish for an immutable entry
pub struct MutEntry<'a> {
    archetype: &'a mut Archetype,
    bundle: usize,
    _phantom: PhantomData<&'a mut EcsManager>,
}

impl<'a> MutEntry<'a> {
    /*
    // Create a mutable entry from the ecs manager and an entity
    pub(crate) fn new(manager: &'a mut EcsManager, entity: Entity) -> Option<Self> {
        let linkings = manager.entities.get(entity)?;
        Some(Self {
            archetype: manager.archetypes.get_mut(&linkings.mask)?,
            bundle: linkings.bundle,
            _phantom: Default::default(),
        })
    }

    // Try to get a component mask
    fn mask<T: Component>(&self) -> Result<Mask, EntryError> {
        let mask = registry::mask::<T>();
        if self.archetype.mask() & mask != mask {
            return Err(EntryError::MissingComponent(registry::name::<T>()));
        }
        Ok(mask)
    }

    // Get an immutable reference to a linked component
    pub fn get<T: Component>(&self) -> Result<&T, EntryError> {
        let mask = self.mask::<T>()?;
        let boxed = self.archetype.storage().get(&mask).unwrap();
        let vec = boxed.as_any().downcast_ref::<Vec<T>>().unwrap();
        Ok(&vec[self.bundle])
    }

    // Get a mutable reference to a linked component
    pub fn get_mut<T: Component>(&mut self) -> Result<&mut T, EntryError> {
        let mask = self.mask::<T>()?;
        self.archetype
            .states()
            .update(self.bundle, |mutated, _| mutated.set(mask.offset(), true));
        self.get_mut_silent()
    }

    // Get a mutable reference to a linked component silently, without triggering a mutation state change
    pub fn get_mut_silent<T: Component>(&mut self) -> Result<&mut T, EntryError> {
        let mask = self.mask::<T>()?;
        let boxed = self.archetype.storage_mut().get_mut(&mask).unwrap();
        let vec = boxed.as_any_mut().downcast_mut::<Vec<T>>().unwrap();
        Ok(&mut vec[self.bundle])
    }

    // Check if a specific component was mutated
    pub fn was_mutated<T: Component>(&self) -> Result<bool, EntryError> {
        let mask = self.mask::<T>()?;
        let mutated = self.archetype.states().get(self.bundle).unwrap();
        Ok(mutated.mutated(mask.offset()))
    }

    // Get a tuple containing the specified components from this entity entry
    pub fn get_mut_layout<'b, Layout: QueryLayout<'b>>(&'b mut self) -> Result<Layout, EntryError> {
        if !Layout::validate() {
            return Err(EntryError::LayoutIntersectingMask);
        }

        let access = Layout::combined();
        let mask = access.shared() | access.unique();
        if !self.archetype.mask().contains(mask) {
            return Err(EntryError::LayoutMissingComponents);
        }

        let ptrs = Layout::try_fetch_ptrs(self.archetype).unwrap();
        Ok(unsafe { Layout::read_as_layout_at(ptrs, self.bundle) })
    }
    */
}
