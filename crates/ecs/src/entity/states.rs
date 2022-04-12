use std::mem::transmute;

use slotmap::Key;

use crate::Entity;

// Each entity state contains the ArchetypalState and 2 extra bits, denoting the entity's validity and if it was accessed
#[repr(u8)]
pub enum ArchetypalState {
    // Nothing's happened to the entity
    Nothing = 0,

    // The entity was added into an archetype
    Validated = 1,

    // The entity will be removed from the archetype
    PendingInvalidation = 2,

    // The entity was moved from one archetype to another
    Moved = 3,
}

// Archetypal State + 2 extra bits
// [    2 bits    ] + validity + accessed
pub struct EntityState(u8);

impl EntityState {
    // Default entity state for new entities
    pub const DEFAULT_STATE: Self = Self::new(ArchetypalState::Nothing, true, true);

    // Creates a new entity state using the archetypal state and it's two extra bits
    pub const fn new(state: ArchetypalState, valid: bool, accessed: bool) -> Self {
        // Bit logic magic
        let state = unsafe { transmute::<ArchetypalState, u8>(state) } << 1;
        let result = state | ((valid as u8) << 1) | (accessed as u8);
        Self(result)
    }

    // Check if an entity is valid
    pub fn is_valid(&self) -> bool {
        (self.0 >> 1) & 1 == 1
    }
    // Check if an entity was accessed using the entries
    pub fn is_accessed(&self) -> bool {
        self.0 % 2 == 1
    }
    // Get the underlying archetypal state region
    pub fn archetypal(&self) -> ArchetypalState {
        unsafe { transmute::<u8, ArchetypalState>(self.0) }
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

const STATES_PER_CHUNK: usize = u16::BITS as usize;
const BITS_PER_CHUNK: usize = u64::BITS as usize;

impl EntityStateSet {
    // Reserve enough capacity to hold "additional" more entity states
    pub fn reserve(&mut self, additional: usize) {
        // Calculate how many new chunks we need
        let old_cap = self.chunks.capacity();
        let new_cap = (self.length + additional) / STATES_PER_CHUNK;
        self.chunks.reserve(new_cap - old_cap)
    }
    // Use a new entity id to see if we should extend the chunks
    pub fn extend_if_needed(&mut self, entity: Entity) {
        // Get the index from the key
        let index = (entity.data().as_ffi() & 0xffff_ffff) as usize;

        // Extend automatically 
        if index > self.length {
            self.length += 1;

            // Add a new chunk if needed
            if (self.length - 1) / STATES_PER_CHUNK > self.length / STATES_PER_CHUNK {
                self.chunks.push(0);
            }
        } 
    }
    // Set an entity state by bitshifting
    // This will return the old state value at that index
    pub fn set(&mut self, state: EntityState, entity: Entity) -> Option<()> {
        // Get the index from the key
        let index = (entity.data().as_ffi() & 0xffff_ffff) as usize;

        // Read the chunk, calculate local element offset, bit offset
        let chunk = self.chunks.get_mut(index / STATES_PER_CHUNK)?;
        let local_offset = index % STATES_PER_CHUNK;
        let bit_offset = local_offset * 4;

        // Write to the chunk
        let state = unsafe { transmute::<EntityState, u8>(state) } as u64;
        *chunk &= !(0b1111 << bit_offset);
        *chunk |= state << bit_offset;
        Some(())
    }
    // Read an entity state by bitshifting
    pub fn get(&self, entity: Entity) -> Option<EntityState> {
        // Get the index from the key
        let index = (entity.data().as_ffi() & 0xffff_ffff) as usize;

        // Read the chunk, calculate local element offset, bit offset
        let chunk = self.chunks.get(index / STATES_PER_CHUNK)?;
        let local_offset = index % STATES_PER_CHUNK;
        let bit_offset = local_offset * 4;

        // Read the state from the chunk
        let state: EntityState = unsafe { transmute::<u8, EntityState>(((*chunk >> index) & 0b1111) as u8) };
        Some(state)
    }
}
