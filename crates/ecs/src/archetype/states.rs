use crate::{Mask, Component, mask};
use std::{cell::RefCell, rc::Rc};

// Component state chunk that contains the component states for a bundle
#[derive(Clone, Copy, Debug)]
pub struct StateRow(Mask, Mask);

impl StateRow {
    // Create a new state row with raw values
    pub fn new(added: Mask, mutated: Mask) -> Self {
        Self(added, mutated)
    }

    // Check if a component (with a specific mask index) was linked to the entity
    pub fn was_added_with_offset(&self, offset: usize) -> bool {
        self.1.get(offset)
    }

    // Check if a component (with a specific mask index) was mutated
    pub fn was_mutated_with_offset(&self, offset: usize) -> bool {
        self.0.get(offset)
    }

    // Check if a component was linked to the entity
    pub fn was_added<T: Component>(&self) -> bool {
        self.was_added_with_offset(mask::<T>().offset())
    }

    // Check if a component was mutated
    pub fn was_mutated<T: Component>(&self) -> bool {
        self.was_mutated_with_offset(mask::<T>().offset())
    }

    // Execute a callback that will modify both masks, and return their old values
    pub fn update(&mut self, f: impl FnOnce(&mut Mask, &mut Mask)) -> StateRow {
        let old = *self;
        f(&mut self.0, &mut self.1);
        old
    }
}