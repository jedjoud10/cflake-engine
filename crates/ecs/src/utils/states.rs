use getset::CopyGetters;

use crate::{archetype::EntityState, Mask, Component, ComponentError, registry};

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

    // Get the current entity state
    pub fn entity(&self) -> EntityState { self.entity }

    // Check if a component was mutated since the start of the frame
    pub fn was_mutated<T: Component>(&self) -> Result<bool, ComponentError> {
        let shifted = registry::mask::<T>()?.0.trailing_zeros();
        Ok((self.components >> shifted) & 1 == 1)
    }
}