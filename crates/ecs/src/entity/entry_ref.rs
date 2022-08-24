use super::Entity;
use crate::{
    registry::{mask, name},
    Archetype, Component, EntityLinkings, EntryError, RefQueryLayout, Scene, StateRow,
};

// Immutable entity entries allow the user to be able to read and get some data about a specific entity
// This data can represent the archetype of the entity or even an immutable reference to a component
pub struct EntryRef<'a> {
    archetype: &'a Archetype,
    linkings: EntityLinkings,
}

impl<'a> EntryRef<'a> {
    // Create an immutable entity entry from the ecs manager and an entity
    pub(crate) fn new(manager: &'a Scene, entity: Entity) -> Option<Self> {
        let linkings = *manager.entities.get(entity)?;
        let archetype = manager.archetypes.get(&linkings.mask()).unwrap();

        Some(Self {
            archetype,
            linkings,
        })
    }

    // Get the entity linkings of the current entity
    pub fn linkings(&self) -> EntityLinkings {
        self.linkings
    }

    // Get an immutable reference to the enitity's archetype
    pub fn archetype(&self) -> &Archetype {
        self.archetype
    }

    // Get an immutable reference to a table
    pub fn table<T: Component>(&self) -> Result<&Vec<T>, EntryError> {
        self.archetype()
            .table::<T>()
            .ok_or_else(|| EntryError::MissingComponent(name::<T>()))
    }

    // Get an immutable reference to a linked component
    pub fn get<T: Component>(&self) -> Result<&T, EntryError> {
        self.table::<T>().map(|vec| &vec[self.linkings().index()])
    }

    // Get the current state row of our entity
    pub fn states(&self) -> StateRow {
        *self
            .archetype()
            .states()
            .borrow()
            .get(self.linkings().index())
            .unwrap()
    }

    // Check if the entity has a component linked to it
    pub fn contains<T: Component>(&self) -> bool {
        self.archetype().mask().contains(mask::<T>())
    }

    // Read certain components from the entry as if they were used in an immutable query
    pub fn as_view<'b, L: RefQueryLayout<'b>>(&self) -> Option<L> {
        let index = self.linkings().index;
        let ptrs = L::prepare(self.archetype)?;
        let layout = unsafe { L::read(ptrs, index) };
        Some(layout)
    }
}
