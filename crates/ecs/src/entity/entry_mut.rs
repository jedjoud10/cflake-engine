use super::Entity;
use crate::{
    add_bundle_unchecked, name, registry::mask, remove_bundle_unchecked, Archetype, ArchetypeSet,
    Bundle, Component, EntityLinkings, EntitySet, QueryLayoutMut, QueryLayoutRef, Scene, StateRow,
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
    pub fn table<T: Component>(&self) -> Option<&Vec<T>> {
        self.archetype().table::<T>()
    }

    // Get a mutable reference to a table
    pub fn table_mut<T: Component>(&mut self) -> Option<&mut Vec<T>> {
        self.archetype_mut().table_mut::<T>()
    }

    // Get an immutable reference to a linked component
    pub fn get<T: Component>(&self) -> Option<&T> {
        self.table::<T>().map(|vec| &vec[self.linkings.index])
    }

    // Get a mutable reference to a linked component, but without triggering a StateRow mutation change
    pub fn get_mut_silent<T: Component>(&mut self) -> Option<&mut T> {
        let i = self.linkings.index;
        self.table_mut::<T>().map(|vec| &mut vec[i])
    }

    // Get a mutable reference to a linked component
    pub fn get_mut<T: Component>(&mut self) -> Option<&mut T> {
        self.table_mut::<T>()?;
        let index = self.linkings().index();
        let states = self.archetype_mut().states_mut();
        let row = &mut states[index];
        row.update(|_added, _removed, mutated| mutated.set(mask::<T>().offset(), true));
        self.get_mut_silent::<T>()
    }

    // Add a new component bundle to the entity, forcing it to switch archetypes
    // This will fail if we try to add some components that were already added
    pub fn insert_bundle<B: Bundle>(&mut self, bundle: B) -> Option<()> {
        assert!(
            B::is_valid(),
            "Bundle is not valid, check the bundle for component collisions"
        );
        add_bundle_unchecked(self.archetypes, self.entity, self.entities, bundle)?;
        self.linkings = self.entities[self.entity];
        Some(())
    }

    // Remove an old component bundle from the entity, forcing it to switch archetypes
    pub fn remove_bundle<B: Bundle>(&mut self) -> Option<B> {
        assert!(
            B::is_valid(),
            "Bundle is not valid, check the bundle for component collisions"
        );
        let bundle = remove_bundle_unchecked(self.archetypes, self.entity, self.entities)?;
        self.linkings = self.entities[self.entity];
        Some(bundle)
    }

    // Get the current state row of our entity
    pub fn states(&self) -> StateRow {
        *self
            .archetype()
            .states()
            .get(self.linkings().index())
            .unwrap()
    }

    // Check if the entity has a component linked to it
    pub fn contains<T: Component>(&self) -> bool {
        self.archetype().mask().contains(mask::<T>())
    }

    // Read certain components from the entry as if they were used in an immutable query
    pub fn as_view<L: for<'s> QueryLayoutRef<'s>>(&self) -> Option<L> {
        // Make sure the layout can be fetched from the archetype
        let combined = L::reduce(|a, b| a | b).both();
        if combined & self.archetype().mask() != combined {
            return None;
        }

        // Fetch the layout from the archetype
        let index = self.linkings().index;
        let ptrs = unsafe { L::ptrs_from_archetype_unchecked(self.archetype()) };
        let layout = unsafe { L::read_unchecked(ptrs, index) };
        Some(layout)
    }

    // Read certain components from the entry as if they were used in an mutable query
    pub fn as_query<L: for<'s> QueryLayoutMut<'s>>(&mut self) -> Option<L> {
        assert!(
            L::is_valid(),
            "Query layout is not valid, check the layout for component collisions"
        );

        // Make sure the layout can be fetched from the archetype
        let access = L::reduce(|a, b| a | b);
        let mutability = access.unique();
        let combined = access.both();
        if combined & self.archetype().mask() != combined {
            return None;
        }

        // Fetch the layout from the archetype
        let index = self.linkings().index;
        let ptrs = unsafe { L::ptrs_from_mut_archetype_unchecked(self.archetype_mut()) };
        let layout = unsafe { L::read_mut_unchecked(ptrs, index) };

        // Update the state row
        self.archetype_mut().states_mut()[index]
            .update(|_, _, update| *update = *update | mutability);
        Some(layout)
    }
}
