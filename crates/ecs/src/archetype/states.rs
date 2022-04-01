mod component;
mod entity;
use std::{sync::Arc, collections::HashMap};

pub use component::*;
pub use entity::*;

use crate::{Mask, MaskHasher};

// Convenience types aliases
type ComponentStates = HashMap<Mask, ComponentMutationsBitfield, MaskHasher>;
type EntityStates = EntityStatesBitfield;

// Struct for organization
#[derive(Default)]
pub(crate) struct ArchetypeStates {
    // The mutated components
    pub components: Arc<ComponentStates>,

    // Bundle entity states
    pub entities: Arc<EntityStates>,

    // Current bundle length
    pub length: usize,
}

impl ArchetypeStates {
    // Create some new archetype states from multiple masks
    pub fn new(masks: impl Iterator<Item = Mask>) -> Self {
        // Collect the component state hashmap
        let components = masks.map(|mask| {
            (mask, ComponentMutationsBitfield::default())
        }).collect::<ComponentStates>();

        // Create a new entity states
        let entities = EntityStates::default();
        
        // Result
        Self {
            components: Arc::new(components),
            entities: Arc::new(entities),
            length: 0,
        }
    }
    // Set the length of the bundles stored in the archetype
    pub fn set_len(&mut self, length: usize) {
        // Bit counts
        let full = u64::BITS as usize;
        let half = full / 2;

        // Check if we should extend
        let extend_entities_bitfield = length.div_ceil(half) > self.length.div_ceil(half); 
        let extend_components_bitfield = length.div_ceil(full) > self.length.div_ceil(full);

        // Extend the bitfields by one AtomicU64 if needed
        if extend_components_bitfield {
            for (_, bitfield) in self.components.iter() {
                bitfield.extend()
            }
        }
        if extend_entities_bitfield { self.entities.extend(); }
    }
    // Get the component mutation state of a specific bundle
    // Get the entity state of a specific bundle
    pub fn get_entity_state(&self, bundle: usize) -> EntityState {
        self.entities.get(bundle)
    }
    // Set the component mutation state of a specific bundle
    pub fn set_component_mutated_state(&mut self, mask: Mask, bundle: usize) {
        let bitfield = self.components.get(&mask).unwrap();
        bitfield.set(bundle);
    }
    // Set the entity state of a specific bundle
    pub fn set_entity_state(&mut self, bundle: usize, state: EntityState) {
        self.entities.set(bundle, state);
    }
}