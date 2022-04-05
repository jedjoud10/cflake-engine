use getset::CopyGetters;

use crate::{archetype::entity::EntityState, Mask, Component, ComponentError, registry};

// The bundle states that can be accessed using a query
pub struct BundleState {
    entity: EntityState,
    components: u64,
}

impl Default for BundleState {
    fn default() -> Self {
        Self { entity: EntityState::Added, components: Default::default() }
    }
}

impl BundleState {
    // Upodate the bundle states
    pub(crate) fn update(&mut self, entity: EntityState, components: u64) {
        self.entity = entity;
        self.components = components;
    } 

    // State getters
    pub fn entity(&self) -> EntityState { self.entity }
    pub fn was_mutated<T: Component>(&self) -> Result<bool, ComponentError> {
        // Get the shift index of the component, so we can search for it's specific mutation bit
        let shifted = registry::mask::<T>()?.0.trailing_zeros();
        Ok((self.components >> shifted) & 1 == 1)
    }
}