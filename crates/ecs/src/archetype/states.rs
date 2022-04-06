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
// Write the value to a bit stored in a u8
fn write_bit_u8(chunk: &mut u8, state: bool, shift: usize) {
    // Set the bit
    if state {
        // Insert
        *chunk |= 1 << shift;
    } else {
        // Remove
        *chunk &= !(1 << shift);
    }
}
// Get the shift counter of a specific unit mask
const fn get_shift_count(mask: Mask) -> usize {
    mask.0.trailing_zeros() as usize
}
// Read a bit stored in a u8
const fn read_bit_u8(chunk: u8, shift: usize) -> bool {
    ((chunk >> shift) & 1) == 1
}

// Constant bit counts
const C64: usize = u64::BITS as usize;
const C32: usize = C64 / 2;
const C8: usize = u8::BITS as usize;

// Contrains an EntityState bitfield and a ComponentMutatedState bitfield
#[derive(Default)]
pub(crate) struct ArchetypeStates {
    // Entity states: 2 bits per entity, can fit 32 states per chunk
    entities: Vec<u64>,

    // Component states: 8 bit per entity (row), can fit 8 states per row, and multiple chunks per collumn
    components: RefCell<Vec<Option<Vec<u8>>>>,

    // Current entity length
    length: usize,
}

impl ArchetypeStates {
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
        for subchunk in self.components.get_mut().iter_mut().filter_map(|x| x.as_mut()) {
            subchunk.push(0);
        }

        // Update the entity's default state
        self.set_entity_state(self.length - 1, EntityState::Added);
    }

    // State setters
    // Set the entity state of a bundle
    pub fn set_entity_state(&mut self, bundle: usize, state: EntityState) {
        // Write the bit to the chunk
        let chunk = self.entities.get_mut(bundle / C32).unwrap();
        write_entity_state_u64(chunk, bundle % C32, state);
    }
    // Set the component state of a bundle using a specific component type
    pub fn set_component_state(&self, bundle: usize, mask: Mask, state: bool) {
        // We might need to extend the main vector
        let shift = get_shift_count(mask);
        let mut components = self.components.borrow_mut();
        if shift >= components.len() {
            // Extend the main vector with many empty chunks
            components.resize_with(shift + 1, || None);
        }

        // Make sure the current collumn is valid
        let collumn = (&mut components[shift / C8]).get_or_insert(vec![0; self.length]);
        let row = collumn.get_mut(bundle).unwrap();

        // Set the bit
        write_bit_u8(row, state, shift % C8);
    }
    // Set all the components states of a specific component type to "state"
    pub fn set_all_component_states(&self, mask: Mask, state: bool) -> Option<()> {
        let shift = get_shift_count(mask);
        let mut borrowed = self.components.borrow_mut();
        let collumn = borrowed.get_mut(shift)?.as_mut().unwrap();

        // Set all the states of a specific bit
        collumn.iter_mut().for_each(|row| write_bit_u8(row, state, shift % C8));
        Some(())
    }

    // Get the state of an entity
    pub fn get_entity_state(&self, bundle: usize) -> Option<EntityState> {
        // Check if the index is valid
        if bundle >= self.length {
            return None;
        }

        // We are 100% sure that we have a valid collumn
        let state = read_entity_state_u64(*self.entities.get(bundle / C32).unwrap(), bundle % C32);
        Some(state)
    }
    // Get the state of a component
    pub fn get_component_state(&self, bundle: usize, mask: Mask) -> Option<bool> {
        // Check if the index is valid
        if bundle >= self.length {
            return None;
        }

        // We are 100% sure that we have a valid collumn
        let shift = get_shift_count(mask);
        let chunk = self.components.borrow()[shift].as_ref().unwrap()[bundle];
        Some(read_bit_u8(chunk, shift % C8))
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

    // Iterate through the entity states using a cached iterator
    pub fn iter_entity_states(&self) -> EntityStatesIter {
        EntityStatesIter {
            states: self.entities.as_slice(),
            chunk: None,
            bundle: 0,
            length: self.length,
        }
    }
    // Iterate through the component states of a unique component mask using a cached iterator
    #[allow(dead_code)]
    pub fn iter_component_states(&self, mask: Mask) -> Option<ComponentStatesIter> {
        let shift = get_shift_count(mask);
        let borrowed = self.components.borrow();

        // Makes sure the collumn is valid (useful for the next step)
        borrowed.get(shift).and_then(|x| x.as_ref())?;
        let collumn = std::cell::Ref::map(borrowed, |b| b.get(shift).unwrap().as_ref().unwrap().as_slice());

        Some(ComponentStatesIter {
            collumn,
            bundle: 0,
            length: self.length,
            mask_shift: shift,
        })
    }
    // Iterate through the component states of all the components at the same time
    pub fn iter_component_states_lanes(&self) -> ComponentFlagLanesIter {
        ComponentFlagLanesIter {
            collumns: self.components.borrow(),
            bundle: 0,
            length: self.length,
        }
    }
}

// Custom cached iterators for better performace
pub struct EntityStatesIter<'a> {
    states: &'a [u64],
    chunk: Option<u64>,
    bundle: usize,
    length: usize,
}

impl<'a> Iterator for EntityStatesIter<'a> {
    type Item = EntityState;

    fn next(&mut self) -> Option<Self::Item> {
        // Check if the index is valid
        if self.bundle >= self.length {
            return None;
        }

        // Invalidate when we reach the start of a new chunk
        if self.bundle % C32 == 0 {
            self.chunk = None
        }

        // Load a chunk into memory when needed
        let chunk = self.chunk.get_or_insert_with(|| self.states[self.bundle / C32]);
        // Super ez
        let res = read_entity_state_u64(*chunk, self.bundle % C32);
        self.bundle += 1;
        Some(res)
    }
}

// Iterates through the states of a specific component type
pub struct ComponentStatesIter<'a> {
    collumn: Ref<'a, [u8]>,
    bundle: usize,
    mask_shift: usize,
    length: usize,
}

impl<'a> Iterator for ComponentStatesIter<'a> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        // Check if the index is valid
        if self.bundle >= self.length {
            return None;
        }

        // Always load the row first
        let row = self.collumn[self.bundle];

        // Super ez
        let res = read_bit_u8(row, self.mask_shift % C8);
        self.bundle += 1;
        Some(res)
    }
}

// Component states lane that contains the states for multiple component types
pub struct FlagLane(u64);

impl FlagLane {
    // Check if a component was mutated since the start of the frame
    pub fn was_mutated<T: Component>(&self) -> Result<bool, ComponentError> {
        let shifted = registry::mask::<T>()?.0.trailing_zeros();
        Ok((self.0 >> shifted) & 1 == 1)
    }
}

// Iterates through all the components states, and pack each unique state into a u64 lane
pub struct ComponentFlagLanesIter<'a> {
    collumns: Ref<'a, Vec<Option<Vec<u8>>>>,
    bundle: usize,
    length: usize,
}

impl<'a> Iterator for ComponentFlagLanesIter<'a> {
    type Item = FlagLane;

    fn next(&mut self) -> Option<Self::Item> {
        // Check if the index is valid
        if self.bundle >= self.length {
            return None;
        }

        // No caching, we must always load a new lane into memory
        let lane = self.collumns.iter().enumerate().fold(0u64, |lane, (collumn_offset, collumn)| {
            // Collumn row -> Global lane
            let row = collumn.as_ref().map(|c| c[self.bundle]).unwrap_or_default() as u64;
            (row << (collumn_offset * 8)) | lane
        });
        self.bundle += 1;
        Some(FlagLane(lane))
    }
}
