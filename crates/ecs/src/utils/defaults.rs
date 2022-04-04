use getset::CopyGetters;

use crate::ArchetypeStates;
use crate::Component;
use crate::ComponentError;
use crate::Entity;
use crate::archetype::entity::EntityState;
use crate::archetype::entity::EntityStatesBitfield;
use crate::registry;

// Some default components
#[derive(Component, CopyGetters)]
pub struct BundleData {
    // Entity linkings
    #[getset(get_copy = "pub")]
    entity: Entity,
    #[getset(get_copy = "pub")]
    bundle: usize,

    // Entity and component states
    states: ArchetypeStates,
}

impl BundleData {
    // Get the current entity state
    pub fn state(&self) -> EntityState {
        self.states.entities.get(self.bundle)
    }
    // Get a component state, if possible
    pub fn was_mutated<T: Component>(&self) -> Option<bool> {
        let bits = self.states.components.get(&registry::mask::<T>().ok()?)?;
        Some(bits.get(self.bundle))
    }
}