use getset::CopyGetters;
use getset::Setters;

use crate::ArchetypeStates;
use crate::Component;
use crate::ComponentError;
use crate::Entity;
use crate::archetype::entity::EntityState;
use crate::archetype::entity::EntityStatesBitfield;
use crate::registry;

// Some default components
#[derive(Default, Component, CopyGetters, Setters)]
pub struct BundleData {
    // Entity linkings
    #[getset(get_copy = "pub", set = "pub(crate)")]
    entity: Entity,
    #[getset(get_copy = "pub", set = "pub(crate)")]
    bundle: usize,

    // Entity and component states
    #[getset(set = "pub(crate)")]
    states: Option<ArchetypeStates>,
}

impl BundleData {
    // Get the current entity state
    pub fn state(&self) -> Option<EntityState> {
        self.states.as_ref().map(|states| states.entities.get(self.bundle))
    }
    // Get a component state, if possible
    pub fn was_mutated<T: Component>(&self) -> Option<bool> {
        self.states.as_ref().and_then(|states| {
            let bits = states.components.get(&registry::mask::<T>().ok()?)?;
            Some(bits.get(self.bundle))
        })
    }
}