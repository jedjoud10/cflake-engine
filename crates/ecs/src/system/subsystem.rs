use crate::{
    component::{LinkedComponents, LinkedComponentsDelta, LinkedComponentsMap},
    entity::EntityKey,
};

use bitfield::Bitfield;
use std::{cell::RefCell, rc::Rc};

// A subsystem can only contain a single component query and a single cbitfield
#[derive(Default)]
pub struct SubSystem {
    pub(crate) cbitfield: Bitfield<u32>,
    pub(super) all: Rc<RefCell<LinkedComponentsMap>>,
    pub(super) delta: Rc<RefCell<LinkedComponentsDelta>>,
}

impl SubSystem {
    // Check if an entity validates our cbitfield
    pub(crate) fn check(&self, cbitfield: Bitfield<u32>) -> bool {
        cbitfield.contains(&self.cbitfield)
    }
    // Add an entity
    pub(crate) fn add(&self, key: EntityKey, linked: LinkedComponents) {
        self.delta.borrow_mut().added.insert(key, linked);
    }
    // Remove an entity
    pub(crate) fn remove(&self, key: EntityKey, linked_components: LinkedComponents) {
        self.delta.borrow_mut().removed.insert(key, linked_components);
    }
}
