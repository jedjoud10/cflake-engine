use crate::{archetype::Archetype, scene::Scene, registry::Component, layout::{Bundle, QueryLayoutRef}};
use super::{Entity, EntityLinkings};

/// Immutable entity entries allow the user to be able to read and get some data about a specific entity.
/// This data can represent the archetype of the entity or even an immutable reference to a component.
pub struct EntryRef<'a> {
    entity: Entity,
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
            entity,
        })
    }

    /// Get the entity handle.
    pub fn entity(&self) -> Entity {
        self.entity
    }

    /// Get the entity linkings of the current entity.
    pub fn linkings(&self) -> EntityLinkings {
        self.linkings
    }

    /// Get an immutable reference to the entity's archetype.
    pub fn archetype(&self) -> &Archetype {
        self.archetype
    }

    /// Get an immutable reference to a linked component.
    pub fn get<T: Component>(&self) -> Option<&T> {
        self.archetype()
            .components::<T>()
            .map(|col| col.get(self.linkings.index).unwrap())
    }

    /// Check if the entity contains the given bundle.
    pub fn contains<B: Bundle>(&self) -> bool {
        let bundle = B::reduce(|a, b| a | b);
        self.archetype().mask().contains(bundle)
    }

    /// Read certain components from the entry as if they were used in an immutable query.
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
}
