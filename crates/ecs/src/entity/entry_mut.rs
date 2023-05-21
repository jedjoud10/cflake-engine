use super::Entity;
use crate::{
    add_bundle, remove_bundle, Archetype, ArchetypeSet, Bundle, Component, EntityLinkings,
    EntitySet, QueryLayoutMut, QueryLayoutRef, RemovedComponents, Scene,
};

// Mutable entity entries allow the user to be able to modify components that are linked to the entity
// They also allow the user to be able to add/remove certain component bundles from the entity
pub struct EntryMut<'a> {
    removed: &'a mut RemovedComponents,
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
        let removed = &mut manager.removed;

        Some(Self {
            removed,
            archetypes,
            entities,
            entity,
            linkings,
        })
    }

    // Get the entity handle
    pub fn entity(&self) -> Entity {
        self.entity
    }

    // Get the entity linkings of the current entity
    pub fn linkings(&self) -> EntityLinkings {
        self.linkings
    }

    // Get an immutable reference to the entity's archetype
    pub fn archetype(&self) -> &Archetype {
        self.archetypes.get(&self.linkings().mask()).unwrap()
    }

    // Get a mutable reference to the entity's archetype
    pub fn archetype_mut(&mut self) -> &mut Archetype {
        self.archetypes.get_mut(&self.linkings().mask()).unwrap()
    }

    // Get an immutable reference to a linked component
    pub fn get<T: Component>(&self) -> Option<&T> {
        self.archetype()
            .components::<T>()
            .map(|col| col.get(self.linkings.index).unwrap())
    }

    // Get a mutable reference to a linked component, but without triggering a StateRow mutation change
    pub fn get_mut_silent<T: Component>(&mut self) -> Option<&mut T> {
        let i = self.linkings.index;
        self.archetype_mut()
            .components_mut::<T>()
            .map(|col| col.get_mut(i).unwrap())
    }

    // Get a mutable reference to a linked component
    pub fn get_mut<T: Component>(&mut self) -> Option<&mut T> {
        let index = self.linkings.index;
        let states = self.archetype_mut().states_mut::<T>()?;
        states.update(index, |flags| flags.modified = true);
        self.get_mut_silent::<T>()
    }

    // Add a new component bundle to the entity, forcing it to switch archetypes
    // This will fail if we try to add some components that were already added
    pub fn insert<B: Bundle>(&mut self, bundle: B) -> Option<()> {
        assert!(
            B::is_valid(),
            "Bundle is not valid, check the bundle for component collisions"
        );

        add_bundle(self.archetypes, self.entity, self.entities, bundle)?;
        self.linkings = self.entities[self.entity];
        Some(())
    }

    // Remove an old component bundle from the entity, forcing it to switch archetypes
    // Returns true when we successfully removed the bundle, false otherwise
    pub fn remove<B: Bundle>(&mut self) -> bool {
        assert!(
            B::is_valid(),
            "Bundle is not valid, check the bundle for component collisions"
        );

        // Move the entity to a new archetype
        let rizz = remove_bundle::<B>(self.archetypes, self.entity, self.entities, self.removed);
        self.linkings = self.entities[self.entity];
        rizz
    }

    // Check if the entity contains the given bundle
    pub fn contains<B: Bundle>(&self) -> bool {
        let bundle = B::reduce(|a, b| a | b);
        self.archetype().mask().contains(bundle)
    }

    // Read certain components from the entry as if they were used in an immutable query
    pub fn as_query<L: QueryLayoutRef>(&self) -> Option<L> {
        // Make sure the layout can be fetched from the archetype
        let search = L::reduce(|a, b| a | b).search();
        if search & self.archetype().mask() != search {
            return None;
        }

        // Fetch the layout from the archetype
        let index = self.linkings().index;
        let ptrs = unsafe { L::ptrs_from_archetype_unchecked(self.archetype()) };
        let layout = unsafe { L::read_unchecked(ptrs, index) };
        Some(layout)
    }

    // Read certain components from the entry as if they were used in an mutable query
    pub fn as_query_mut<L: QueryLayoutMut>(&mut self) -> Option<L> {
        assert!(
            L::is_valid(),
            "Query layout is not valid, check the layout for component collisions"
        );

        // Make sure the layout can be fetched from the archetype
        let access = L::reduce(|a, b| a | b);
        let search = access.search();
        if search & self.archetype().mask() != search {
            return None;
        }

        // Fetch the layout from the archetype
        let index = self.linkings().index;
        let ptrs = unsafe { L::ptrs_from_mut_archetype_unchecked(self.archetype_mut()) };
        let layout = unsafe { L::read_mut_unchecked(ptrs, index) };

        // Get a mask of changed components from the archetype
        let archetype = self.archetype_mut();
        let mutability = archetype.mask() & access.unique();

        // Update the states based on the layout mask
        for unit in mutability.units() {
            let table = archetype.table_mut();
            let states = table.get_mut(&unit).unwrap().states_mut();
            states.update(index, |flags| flags.modified = true);
        }

        Some(layout)
    }
}
