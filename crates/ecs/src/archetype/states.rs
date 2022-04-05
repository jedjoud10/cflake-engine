pub mod component;
pub mod entity;
use crate::{Mask, MaskHasher};
use component::*;
use entity::*;
use std::{collections::HashMap, sync::Arc};

// Convenience types aliases
pub(crate) type ComponentStates = HashMap<Mask, ComponentMutationsBitfield, MaskHasher>;
pub(crate) type EntityStates = EntityStatesBitfield;

// Struct for organization
#[derive(Default)]
pub(crate) struct ArchetypeStates {
    // The mutated components
    pub components: Arc<ComponentStates>,

    // Bundle entity states
    pub entities: EntityStates,

    // Current bundle length
    pub length: usize,
}

impl ArchetypeStates {
    // Create some new archetype states from multiple masks
    pub fn new(masks: impl Iterator<Item = Mask>) -> Self {
        // Collect the component state hashmap
        let components = Arc::new(masks.map(|mask| (mask, ComponentMutationsBitfield::default())).collect::<ComponentStates>());

        // Create a new entity states
        let entities = EntityStates::default();

        // Result
        Self { components, entities, length: 0 }
    }
    // Set the length of the bundles stored in the archetype
    pub fn set_len(&self, length: usize) {
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
        if extend_entities_bitfield {
            self.entities.extend();
        }
    }
}
