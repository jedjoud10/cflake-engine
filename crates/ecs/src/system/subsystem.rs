use crate::{
    component::{ComponentQueryParameters, LinkedComponents, LinkedComponentsDelta, LinkedComponentsMap},
    entity::EntityKey,
};
use ahash::AHashMap;
use bitfield::Bitfield;
use std::cell::RefCell;

// A subsystem can only contain a single component query and a single cbitfield
pub struct SubSystem {
    pub(crate) cbitfield: Bitfield<u32>,
    pub(super) all: LinkedComponentsMap,
    pub(super) delta: LinkedComponentsDelta,
}

impl SubSystem {
    // Create a new subsystem using some query parameters
    pub(crate) fn new(params: ComponentQueryParameters) -> Self {
        Self {
            cbitfield: params.cbitfield,
            all: Default::default(),
            delta: Default::default(),
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
        self.all.insert(key, linked_components);
        self.delta.added.insert(key, cloned);
    }
    // Remove an entity
    pub(crate) fn remove(&mut self, key: EntityKey, linked_components: LinkedComponents) {
        if self.all.contains_key(&key) {
            self.all.remove(&key);
            self.delta.removed.insert(key, linked_components);
        }
    }
}
