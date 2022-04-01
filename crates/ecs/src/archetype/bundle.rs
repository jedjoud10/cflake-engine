use std::{sync::Arc, collections::HashMap};

use getset::Getters;

use crate::{Entity, EntityStatesBitfield, Mask, ComponentMutationsBitfield, EcsManager, Archetype};


// A single archetype bundle identifier
#[derive(Getters)]
pub struct ArchetypeBundle {
    // Entity
    #[getset(get = "pub")]
    entity: Entity,

    // Index
    #[getset(get = "pub")]
    index: usize,

    // Entity states
    #[getset(get = "pub(crate)")]
    states: Arc<EntityStatesBitfield>,

    // Cloned storaes so we can check the mutated states
    #[getset(get = "pub(crate)")]
    cloned: Arc<HashMap<Mask, ComponentMutationsBitfield>>,
}

impl ArchetypeBundle {
    // Create a new archetype bundle
    pub fn new(index: usize, entity: Entity, archetype: &Archetype) -> Self {
        Self {
            entity,
            index,
            states: archetype.states(),
            cloned: todo!(),
        }
    }
}
