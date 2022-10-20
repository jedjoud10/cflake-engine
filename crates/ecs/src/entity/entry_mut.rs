use super::Entity;
use crate::{
    add_bundle_unchecked,
    registry::{mask},
    remove_bundle_unchecked, Archetype, ArchetypeSet, Bundle, Component, EntityLinkings, EntitySet,
    EntryError, Scene, StateRow, name,
};

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
    pub(crate) fn new(manager: &'a mut Scene, entity: Entity) -> Option<Self> {
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
        self.archetype()
            .table::<T>()
            .ok_or_else(|| EntryError::MissingComponent(name::<T>()))
    }

    // Get a mutable reference to a table
    pub fn table_mut<T: Component>(&mut self) -> Result<&mut Vec<T>, EntryError> {
        self.archetype_mut()
            .table_mut::<T>()
            .ok_or_else(|| EntryError::MissingComponent(name::<T>()))
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
        let states = self.archetype_mut().states();
        let mut slice = states.borrow_mut();
        let row = &mut slice[index];
        row.update(|_added, _removed, mutated| mutated.set(mask::<T>().offset(), true));
        let val = self.get_mut_silent::<T>().unwrap();
        Ok(val)
    }

    // Get a mutable reference to a linked component, but without triggering a StateRow mutation change
    pub fn get_mut_silent<T: Component>(&mut self) -> Result<&mut T, EntryError> {
        let index = self.linkings().index();
        self.table_mut::<T>().map(|vec| &mut vec[index])
    }

    // Add a new component bundle to the entity, forcing it to switch archetypes
    pub fn insert_bundle<B: Bundle>(&mut self, bundle: B) -> Option<()> {
        add_bundle_unchecked(self.archetypes, self.entity, self.entities, bundle)?;
        self.linkings = self.entities[self.entity];
        Some(())
    }

    // Remove an old component bundle from the entity, forcing it to switch archetypes
    pub fn remove_bundle<B: Bundle>(&mut self) -> Option<B> {
        let bundle = remove_bundle_unchecked(self.archetypes, self.entity, self.entities)?;
        self.linkings = self.entities[self.entity];
        Some(bundle)
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

    /*
    // Read certain components from the entry as if they were used in an immutable query
    pub fn as_view<'b, L: RefQueryLayout<'b>>(&self) -> Option<L> {
        let index = self.linkings().index;
        let ptrs = L::prepare(self.archetype())?;
        let layout = unsafe { L::read(ptrs, index) };
        Some(layout)
    }

    // Read certain components from the entry as if they were used in an mutable query
    pub fn as_query<'s: 'l, 'l, L: MutQueryLayout<'s, 'l>>(&mut self) -> Option<L> {
        let index = self.linkings().index;
        let mut slices = L::prepare(self.archetype_mut())?;
        let layout = L::read(&mut slices, index);
        Some(layout)
    }
    */
}
