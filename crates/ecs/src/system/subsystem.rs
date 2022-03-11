use std::cell::RefCell;
use ahash::AHashMap;
use bitfield::Bitfield;
use crate::{component::{LinkedComponents, ComponentQueryParameters}, entity::EntityKey};

// A subsystem can only contain a single component query and a single cbitfield
pub struct SubSystem {    
    pub(crate) cbitfield: Bitfield<u32>,
    pub(super) linked_components: AHashMap<EntityKey, LinkedComponents>,
    // Added, Removed
    pub(super) added: AHashMap<EntityKey, LinkedComponents>,
    pub(super) removed: AHashMap<EntityKey, LinkedComponents>,
}

impl SubSystem {
    // Create a new subsystem using some query parameters
    pub(crate) fn new(params: ComponentQueryParameters) -> Self {
        Self {
            cbitfield: params.cbitfield,
            linked_components: Default::default(),
            added: Default::default(),
            removed: Default::default()
        }
    }
    // Check if an entity validates our cbitfield
    pub(crate) fn check(&self, cbitfield: Bitfield<u32>) -> bool {
        cbitfield.contains(&self.cbitfield)
    }
    // Add an entity
    pub(crate) fn add(&mut self, key: EntityKey, linked_components: LinkedComponents) {
        let cloned = LinkedComponents {
            components: linked_components.components.clone(),
            mutated_components: linked_components.mutated_components.clone(),
            linked: linked_components.linked.clone(),
            key,
        };
        self.linked_components.insert(key, linked_components);
        self.added.insert(key, cloned);
    }
    // Remove an entity
    pub(crate) fn remove(&mut self, key: EntityKey, linked_components: LinkedComponents) {
        if self.linked_components.contains_key(&key) {
            self.linked_components.remove(&key);
            self.removed.insert(key, linked_components);
        }
    }
}