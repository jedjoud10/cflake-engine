use super::Entity;
use crate::{
    Archetype, Component, EntityLinkings,
    QueryLayoutRef, Scene, Bundle,
};

// Immutable entity entries allow the user to be able to read and get some data about a specific entity
// This data can represent the archetype of the entity or even an immutable reference to a component
pub struct EntryRef<'a> {
    archetype: &'a Archetype,
    linkings: EntityLinkings,
}

impl<'a> EntryRef<'a> {
    // Create an immutable entity entry from the ecs manager and an entity
    pub(crate) fn new(
        manager: &'a Scene,
        entity: Entity,
    ) -> Option<Self> {
        let linkings = *manager.entities.get(entity)?;
        let archetype =
            manager.archetypes.get(&linkings.mask()).unwrap();

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

    // Get an immutable reference to a tableStateRow
    pub fn table<T: Component>(&self) -> Option<&Vec<T>> {
        self.archetype().components::<T>()
    }

    // Get an immutable reference to a linked component
    pub fn get<T: Component>(&self) -> Option<&T> {
        self.table::<T>().map(|vec| &vec[self.linkings.index])
    }

    // Check if the entity contains the given bundle
    pub fn contains<B: Bundle>(&self) -> bool {
        let bundle = B::reduce(|a, b| a | b);
        self.archetype().mask().contains(bundle)
    }

    // Read certain components from the entry as if they were used in an immutable query
    pub fn as_query<L: for<'s> QueryLayoutRef<'s>>(
        &self,
    ) -> Option<L> {
        // Make sure the layout can be fetched from the archetype
        let combined = L::reduce(|a, b| a | b).both();
        if combined & self.archetype().mask() != combined {
            return None;
        }

        // Fetch the layout from the archetype
        let index = self.linkings().index;
        let ptrs = unsafe {
            L::ptrs_from_archetype_unchecked(self.archetype())
        };
        let layout = unsafe { L::read_unchecked(ptrs, index) };
        Some(layout)
    }
}
