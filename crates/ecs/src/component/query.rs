use super::{registry, Component, LinkedComponents};
use crate::entity::EntityKey;
use ahash::AHashMap;
use bitfield::Bitfield;
use std::{
    cell::RefMut,
    ops::{Deref, DerefMut},
};
// A struct full of LinkedComponents that we send off to update in parallel
// This will use the components data given by the world to run all the component updates in PARALLEL
// The components get mutated in parallel, though the system is NOT stored on another thread
pub(crate) type LinkedComponentsMap = AHashMap<EntityKey, LinkedComponents>;

// Added/removed
#[derive(Default)]
pub struct LinkedComponentsDelta {
    pub added: LinkedComponentsMap,
    pub removed: LinkedComponentsMap,
}

// Some query parameters for a single component query
#[derive(Default)]
pub struct ComponentQueryParameters {
    pub(crate) cbitfield: Bitfield<u32>,
}

impl ComponentQueryParameters {
    // This component query shall use components that validate this bitfield
    pub fn link<U: Component + 'static>(mut self) -> Self {
        let c = registry::get_component_bitfield::<U>();
        self.cbitfield = self.cbitfield.add(&c);
        self
    }
}

// A single component query that contains the added/removed components, alongside all the components
pub struct ComponentQuery<'a> {
    pub all: RefMut<'a, LinkedComponentsMap>,
    pub delta: RefMut<'a, LinkedComponentsDelta>,
}

// A component query set that contains multiple queries that can be fetched from the subsystems of a specific system
pub struct ComponentQuerySet<'subsystem> {
    // Total
    pub(crate) queries: Vec<ComponentQuery<'subsystem>>,
}

impl<'subsystem> ComponentQuerySet<'subsystem> {
    // Get a specific component query using it's subsystem index
    pub fn get(&mut self, index: usize) -> &mut ComponentQuery<'subsystem> {
        self.queries.get_mut(index).unwrap()
    }
}
