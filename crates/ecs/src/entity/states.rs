use std::mem::transmute;

use slotmap::Key;

use crate::Entity;

// Each entity contains 2 simple bits depicting it's state in the archetype
// [moved, valid]
pub struct EntityState(u8);

impl EntityState {
    // Creates a new entity state using it's bits
    pub const fn new(moved: bool, valid: bool) -> Self {
        let a = (moved as u8) << 1;
        let b = (valid as u8);
        Self(a | b)
    }

    // Get the states by bitshifting
    pub const fn moved(&self) -> bool {
        (self.0 >> 1) == 1
    }
    pub const fn valid(&self) -> bool {
        self.0 & 1 == 1
    }
}

// The state for each entity that is contained within the manager
#[derive(Default)]
pub struct EntityStateSet {
    // Each chunk can hold 16 entity states, since each entity state is 4 bits
    chunks: Vec<u64>,

    // Number of entity states stored in total
    length: usize,
}

const BITS_PER_STATE: usize = 2;
const STATES_PER_CHUNK: usize = u64::BITS as usize / BITS_PER_STATE;
const BITS_PER_CHUNK: usize = 64;

impl EntityStateSet {
    // Extend the entity states by a specific amount of new elements and fill the states with a given state
    pub fn extend_by(&mut self, additional: usize, state: EntityState) {
        // Calculate how many new chunks we need
        let old_len = self.chunks.len();
        let new_len = ((self.length + additional) / STATES_PER_CHUNK) + 1;

        // Create a default states u64
        let def = (0..BITS_PER_CHUNK).into_iter().step_by(BITS_PER_STATE).fold(0u64, |a, offset| {
            // Get the 4 bits and bitshift them, then combine them
            let bits = state.0 as u64;
            a | (bits << offset)
        });

        // Default chunk iterator
        let iter = (0..(new_len - old_len)).into_iter().map(|_| def);
        self.chunks.extend(iter);
        self.length += additional;
    }
    // Use a new entity id to see if we should extend the chunks
    pub fn extend_if_needed(&mut self, entity: Entity) {
        // Get the index from the key
        let index = (entity.data().as_ffi() & 0xffff_ffff) as usize;

        // Extend automatically
        if index >= self.length {
            self.length += 1;

            // Add a new chunk if needed
            if (self.length - 1) / STATES_PER_CHUNK > self.length / STATES_PER_CHUNK {
                self.chunks.push(0);
            }
        }
    }
    // Update an entity state using a callback
    pub fn update(&mut self, entity: Entity, callback: impl FnOnce(EntityState) -> EntityState) -> Option<()> {
        // Get the index from the key
        let index = (entity.data().as_ffi() & 0xffff_ffff) as usize;

        // Read the chunk, calculate local element offset, bit offset
        let chunk = self.chunks.get_mut(index / STATES_PER_CHUNK)?;
        let local_offset = index % STATES_PER_CHUNK;
        let bit_offset = local_offset * 4;

        // Write to the chunk by calling the function
        let old_bits = ((*chunk >> bit_offset) & 0b1111) as u8;
        let old_state = unsafe { transmute::<u8, EntityState>(old_bits) };
        let new_state = unsafe { transmute::<EntityState, u8>(callback(old_state)) } as u64;
        *chunk &= !(0b1111 << bit_offset);
        *chunk |= new_state << bit_offset;
        Some(())
    }
    // Set an entity state directly
    pub fn set(&mut self, entity: Entity, state: EntityState) -> Option<()> {
        self.update(entity, |_| state)
    }
    // Read an entity state by bitshifting
    pub fn get(&self, entity: Entity) -> Option<EntityState> {
        // Get the index from the key
        let index = (entity.data().as_ffi() & 0xffff_ffff) as usize;

        // Read the chunk, calculate local element offset, bit offset
        let chunk = self.chunks.get(index / STATES_PER_CHUNK).unwrap_or_else(|| panic!("{index}"));
        let local_offset = index % STATES_PER_CHUNK;
        let bit_offset = local_offset * 4;

        // Read the state from the chunk
        let state: EntityState = unsafe { transmute::<u8, EntityState>(((*chunk >> bit_offset) & 0b1111) as u8) };
        Some(state)
    }
}
