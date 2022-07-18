use std::marker::PhantomData;
use super::Entity;
use crate::{registry::{self, mask, name}, Archetype, Component, EcsManager, EntryError, Mask, ArchetypeSet, EntitySet, StateRow, EntityLinkings};

// Mutable entity entries allow the user to be able to modify components that are linked to the entity
// They also allow the user to be able to add/remove certain component bundles from the entity
pub struct EntryMut<'a> {
    archetypes: &'a mut ArchetypeSet,
    entities: &'a mut EntitySet,
    entity: Entity,
    linkings: EntityLinkings,
}

impl<'a> EntryMut<'a> {
    // Create a mutable entry from the ecs manager and an entity
    pub(crate) fn new(manager: &'a mut EcsManager, entity: Entity) -> Option<Self> {
        let linkings = *manager.entities.get(entity)?;
        let archetypes = &mut manager.archetypes;
        let entities = &mut manager.entities;

        Some(Self {
            archetypes,
            entities,
            entity,
            linkings,
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
        self.archetype().table::<T>().ok_or_else(|| EntryError::MissingComponent(name::<T>()))
    }

    // Get a mutable reference to a table
    pub fn table_mut<T: Component>(&mut self) -> Result<&mut Vec<T>, EntryError> {
        self.archetype_mut().table_mut::<T>().ok_or_else(|| EntryError::MissingComponent(name::<T>()))
    }

    // Get an immutable reference to a linked component
    pub fn get<T: Component>(&self) -> Result<&T, EntryError> {
        self.table::<T>().map(|vec| &vec[self.linkings().index()])
    }

    // Get a mutable reference to a linked component
    pub fn get_mut<T: Component>(&mut self) -> Result<&mut T, EntryError> {
        if !self.contains::<T>() {
            return Err(EntryError::MissingComponent(name::<T>()));
        }

        let index = self.linkings().index();
        let states = self.archetype_mut().states_mut();
        let row = &mut states[index];
        row.update(|added, mutated| mutated.set(mask::<T>().offset(), true));
        let val = self.get_mut_silent::<T>().unwrap();        
        Ok(val)
    }

    // Get a mutable reference to a linked component, but without triggering a StateRow mutation change
    pub fn get_mut_silent<T: Component>(&mut self) -> Result<&mut T, EntryError> {
        let index = self.linkings().index();
        self.table_mut::<T>().map(|vec| &mut vec[index])
    }

    // Add a new component bundle to the entity, forcing it to switch archetypes
    // Remove an old component bundle from the entity, forcing it to switch archetypes
    
    // Get the current state row of our entity
    pub fn states(&self) -> StateRow {
        *self.archetype().states().get(self.linkings().index()).unwrap()
    }

    // Check if the entity has a component linked to it
    pub fn contains<T: Component>(&self) -> bool {
        self.archetype().mask().contains(mask::<T>())
    }

    // Get a tuple containing the specified components from this entity entry
    /*
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
