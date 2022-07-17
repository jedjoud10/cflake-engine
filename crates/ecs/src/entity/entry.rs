use std::marker::PhantomData;

use super::Entity;
use crate::{registry::{self, mask, name}, Archetype, Component, EcsManager, EntryError, Mask, QueryLayout, ArchetypeSet, EntitySet, StateRow, EntityLinkings};

// An entity entry that we can use to access multiple components on a single entity
// This will take an immutable reference of the ECS manager, so we cannot mutate any of the underlying entity components
pub struct Entry<'a> {
    archetype: &'a Archetype,
    linkings: EntityLinkings,
    _phantom: PhantomData<&'a EcsManager>,
}

impl<'a> Entry<'a> {
    // Create an immutable entity entry from the ecs manager and an entity
    pub fn new(manager: &'a EcsManager, entity: Entity) -> Option<Self> {
        let linkings = manager.entities.get(entity)?;
        Some(Self {
            archetype: manager.archetypes.get(&linkings.mask)?,
            linkings,
            _phantom: Default::default(),
        })
    }

    // Get the entity linkings of the current entity
    pub fn linkings(&self) -> EntityLinkings {
        self.linkings
    }

    // Get an immutable reference to the enitity's archetype
    pub fn archetype(&self) -> &Archetype {
        &self.archetype
    }
    
    // Get a mutable reference to the enitity's archetype
    pub fn archetype_mut(&mut self) -> &mut Archetype {
        &mut self.archetype
    }

    // Get an immutable reference to a table
    pub fn table<T: Component>(&self) -> Result<&Vec<T>, EntryError> {
        self.archetype().table::<T>().ok_or(EntryError::MissingComponent(name::<T>()))
    }

    // Get an immutable reference to a linked component
    pub fn get<T: Component>(&self) -> Result<&T, EntryError> {
        self.table::<T>().map(|vec| &vec[self.linkings.index()])
    }
    
    // Get the current state row of our entity
    pub fn states(&self) -> StateRow {
        self.archetype().states().get(self.linkings.index()).unwrap()
    }

    // Check if the entity has a component linked to it
    pub fn contains<T: Component>(&self) -> bool {
        self.archetype().mask().contains(mask::<T>())
    }
}

// An entity entry that we can use to access multiple components on a single entity
// This will take a mutable reference of the ECS manager. Use Entry instead if you wish for an immutable entry
pub struct EntryMut<'a> {
    archetypes: &'a mut ArchetypeSet,
    entities: &'a mut EntitySet,
    entity: Entity,
    linkings: EntityLinkings,
    _phantom: PhantomData<&'a mut EcsManager>,
}

impl<'a> EntryMut<'a> {
    // Create a mutable entry from the ecs manager and an entity
    pub fn new(manager: &'a mut EcsManager, entity: Entity) -> Option<Self> {
        Some(Self {
            archetypes: &mut manager.archetypes,
            entities: &mut manager.entities,
            entity,
            linkings: manager.entities.get(entity)?,
            _phantom: Default::default(),
        })
    }
    
    // Get the entity linkings of the current entity
    pub fn linkings(&self) -> EntityLinkings {
        self.linkings
    }

    // Get an immutable reference to the enitity's archetype
    pub fn archetype(&self) -> &Archetype {
        self.archetypes.get(&self.linkings().mask()).unwrap()
    }
    
    // Get a mutable reference to the enitity's archetype
    pub fn archetype_mut(&mut self) -> &mut Archetype {
        self.archetypes.get_mut(&self.linkings().mask()).unwrap()
    }

    // Get an immutable reference to a table
    pub fn table<T: Component>(&self) -> Result<&Vec<T>, EntryError> {
        self.archetype().table::<T>().ok_or(EntryError::MissingComponent(name::<T>()))
    }

    // Get a mutable reference to a table
    pub fn table_mut<T: Component>(&mut self) -> Result<&mut Vec<T>, EntryError> {
        self.archetype_mut().table_mut::<T>().ok_or(EntryError::MissingComponent(name::<T>()))
    }

    // Get an immutable reference to a linked component
    pub fn get<T: Component>(&self) -> Result<&T, EntryError> {
        self.table::<T>().map(|vec| &vec[self.linkings().index()])
    }

    // Get a mutable reference to a linked component
    pub fn get_mut<T: Component>(&mut self) -> Result<&mut T, EntryError> {
        let states = self.archetype_mut().states();
        states.update()
        Ok(&mut table[self.linkings().index()])
    }

    // Get a mutable reference to a linked component, but without triggering a StateRow mutation change
    pub fn get_mut_silent(&mut self) -> Result<&mut T, EntryError> {
        self.table_mut::<T>().map(|vec| &mut vec[self.index])
    }


    // Add a new component bundle to the entity, forcing it to switch archetypes
    // Remove an old component bundle from the entity, forcing it to switch archetypes
    
    // Get the current state row of our entity
    pub fn states(&self) -> StateRow {
        self.archetype.states().get(self.index).unwrap()
    }

    // Check if the entity has a component linked to it
    pub fn contains<T: Component>(&self) -> bool {
        self.archetype.mask().contains(mask::<T>())
    }

    // Try to get a component mask
    fn mask<T: Component>(&self) -> Result<Mask, EntryError> {
        let mask = mask::<T>();
        if self.archetype.mask() & mask != mask {
            return Err(EntryError::MissingComponent(name::<T>()));
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

        self.get_mut_silent()
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
