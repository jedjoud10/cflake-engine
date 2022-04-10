use std::cell::{Ref, RefCell};

use crate::{registry, Component, ComponentError, Mask};

// Entity state of a stored bundle in the archetype
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntityState {
    // Nothing happened to the entity
    None = 0,

    // The entity just got added to the archetype
    Added = 1,

    // The bundled entity is pending for removal, and it will be removed next frame
    PendingForRemoval = 2,
}

// Write entity state to a u64 (offset is the element shift, not raw bit shift)
fn write_entity_state_u64(chunk: &mut u64, index: usize, state: EntityState) {
    let masked = *chunk & !(0b11 << (index * 2));
    let shifted = unsafe { std::mem::transmute::<EntityState, u8>(state) as u64 } << (index * 2);
    let inserted = masked | shifted;
    *chunk = inserted;
}
// Read entity state from a u64 (offset is the element shift, not raw bit shift)
fn read_entity_state_u64(chunk: u64, index: usize) -> EntityState {
    let filtered = (chunk >> (index * 2)) & 0b11;
    unsafe { std::mem::transmute::<u8, EntityState>(filtered as u8) }
}
// Write the value to a bit stored in a u64
fn write_component_state_u64(lane: &mut u64, state: bool, shift: usize) {
    // Set the bit
    if state {
        // Insert
        *lane |= 1 << shift;
    } else {
        // Remove
        *lane &= !(1 << shift);
    }
}
// Get the shift counter of a specific unit mask
const fn get_shift_count(mask: Mask) -> usize {
    mask.0.trailing_zeros() as usize
}
// Read a bit stored in a u64
const fn read_component_state_u64(lane: u64, shift: usize) -> bool {
    ((lane >> shift) & 1) == 1
}

// Constant bit counts
const C64: usize = u64::BITS as usize;
const C32: usize = C64 / 2;
const C8: usize = u8::BITS as usize;

// Contrains an EntityState bitfield and a ComponentMutatedState bitfield
#[derive(Default)]
pub(crate) struct ArchetypeStates {
    entities: Vec<u64>,
    components: RefCell<Vec<u64>>,
    length: usize,
}

impl ArchetypeStates {
    // Reserve enough space to fit "additional"  more elements
    pub fn reserve(&mut self, additional: usize) {
        self.components.get_mut().reserve(additional);
    }

    // Push an entity into the states
    pub fn push(&mut self) {
        // Needed for comparison
        let old = self.length;
        let new = self.length + 1;
        self.length = new;

        // Check if we need to add a new entity states chunk
        if new.div_ceil(C32) > old.div_ceil(C32) {
            self.entities.push(0);
        }

        // Always add the new component states
        self.components.get_mut().push(0);

        // Update the entity's default state
        self.set_entity_state(old, EntityState::Added);
    }

    // State setters
    // Set the entity state of a bundle
    pub fn set_entity_state(&mut self, bundle: usize, state: EntityState) {
        let chunk = &mut self.entities[bundle / C32];
        write_entity_state_u64(chunk, bundle % C32, state);
    }
    // Set the component state of a bundle using a specific component type
    pub fn set_component_state(&self, bundle: usize, mask: Mask, state: bool) {
        let shift = get_shift_count(mask);
        let mut components = self.components.borrow_mut();
        write_component_state_u64(&mut components[bundle], state, shift % C8);
    }
    // Set all the components states of a specific component type to "state"
    pub fn set_all_component_states(&self, mask: Mask, state: bool) {
        let shift = get_shift_count(mask);
        let mut borrowed = self.components.borrow_mut();
        borrowed.iter_mut().for_each(|lane| write_component_state_u64(lane, state, shift));
    }
    // Get the state of an entity
    pub fn get_entity_state(&self, bundle: usize) -> EntityState {
        read_entity_state_u64(*self.entities.get(bundle / C32).unwrap(), bundle % C32)
    }
    // Get the state of a component
    pub fn get_component_state(&self, bundle: usize, mask: Mask) -> Option<bool> {
        let shift = get_shift_count(mask);
        let lane = *self.components.borrow().get(bundle)?;
        Some(read_component_state_u64(lane, shift))
    }

    // Reset all the entity states to their default value
    pub fn reset_entity_states(&mut self) {
        for x in self.entities.iter_mut() {
            *x = 0
        }
    }
    // Reset all the component states (of a specific component type) to their default value
    pub fn reset_component_states(&mut self, mask: Mask) {
        self.set_all_component_states(mask, false);
    }
}