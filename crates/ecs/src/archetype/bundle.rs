use std::{sync::Arc, collections::HashMap};

use getset::Getters;

use crate::{Entity, EntityStatesBitfield, Mask, ComponentMutationsBitfield, EcsManager, Archetype, ArchetypeStates};


// A single archetype bundle identifier
#[derive(Getters)]
pub struct ArchetypeBundle {
    // The current entity that is linked to this bundle
    #[getset(get = "pub")]
    entity: Entity,

    // Current bundle index of this archetype bundle
    #[getset(get = "pub")]
    index: usize,

    // States (Component and entity states)
    #[getset(get = "pub(crate)")]
    states: Arc<ArchetypeStates>,
}

impl ArchetypeBundle {
    // Create a new archetype bundle
    pub fn new(index: usize, entity: Entity, archetype: &Archetype) -> Self {
        Self {
            entity,
            index,
            states: archetype.states().clone(),
        }
    }
}
