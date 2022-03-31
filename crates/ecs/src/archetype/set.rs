use super::{Archetype, ArchetypeId, NoHash};
use crate::component::ComponentLayout;
use std::collections::HashMap;

// The archetype hashmap
type ArchetypeHashMap = HashMap<ArchetypeId, Archetype, NoHash>;

// Archetype set
#[derive(Default)]
pub struct ArchetypeSet {
    // Multiple archetypes
    archetypes: ArchetypeHashMap,
}

impl ArchetypeSet {
    // Register an archetype into the set using a layout
    pub fn register(&mut self, layout: ComponentLayout) -> ArchetypeId {
        // No need to add a new archetype if it exists already
        let id = ArchetypeId(layout.mask);
        if self.archetypes.contains_key(&id) {
            return id;
        }

        // Insert a new archetype with the specified layout
        self.archetypes.insert(ArchetypeId(layout.mask), Archetype::new(layout));
        id
    }

    // Get an archetype immutably
    pub fn get(&self, id: ArchetypeId) -> Option<&Archetype> {
        self.archetypes.get(&id)
    }
    // Get an archetype mutably
    pub fn get_mut(&mut self, id: ArchetypeId) -> Option<&mut Archetype> {
        self.archetypes.get_mut(&id)
    }

    // Iterators
    pub fn iter(&self) -> impl Iterator<Item = (&ArchetypeId, &Archetype)> {
        self.archetypes.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&ArchetypeId, &mut Archetype)> {
        self.archetypes.iter_mut()
    }
}
