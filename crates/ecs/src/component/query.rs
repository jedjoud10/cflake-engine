use super::{LinkedComponents, Component, registry};
use crate::{entity::EntityKey};
use ahash::AHashMap;
use bitfield::Bitfield;
use std::{ops::{Deref, DerefMut}, cell::RefMut};
// A struct full of LinkedComponents that we send off to update in parallel
// This will use the components data given by the world to run all the component updates in PARALLEL
// The components get mutated in parallel, though the system is NOT stored on another thread
type LinkedComponentsMap = AHashMap<EntityKey, LinkedComponents>;

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

// A component query set that contains multiple queries that can be fetched from the subsystems of a specific system
pub struct ComponentQuerySet<'a> {
    pub(crate) queries: Vec<Option<RefMut<'a, LinkedComponentsMap>>>
}

impl<'a> ComponentQuerySet<'a> {
    // Get a specific component query using it's subsystem index
    // When you fetch a component query, you cannot re-fetch it
    pub fn fetch(&mut self, index: usize) -> Option<RefMut<'a, LinkedComponentsMap>> {
        let opt = self.queries.get_mut(index)?;
        opt.take()
    }
}