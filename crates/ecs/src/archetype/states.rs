use crate::Mask;

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

// Write entity state to a u64
fn write_entity_state_u64(chunk: &mut u64, shift: usize, state: EntityState) {
    let masked = *chunk & !(0b11 << shift);
    let inserted = masked | unsafe { std::mem::transmute::<EntityState, u8>(state) as u64 };
    *chunk = inserted;
}
// Read entity state from a u64
fn read_entity_state_u64(chunk: u64, shift: usize) -> EntityState {
    let filtered = (chunk >> shift) & 0b11;
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
// Read a bit stored in a u8
const fn read_bit_u8(chunk: u8, shift: usize) -> bool {
    ((chunk >> shift) & 1) == 1
}

// Consts
const HALF: usize = (u64::BITS as usize) / 2;
const MINI: usize = u8::BITS as usize;

// Contrains an EntityState bitfield and a ComponentMutatedState bitfield
#[derive(Default)]
pub struct ArchetypeStates {
    // Entity states: 2 bits per entity, can fit 32 states per chunk
    entities: Vec<u64>,

    // Component states: 8 bit per entity (row), can fit 8 states per row, and multiple chunks per collumn
    components: Vec<Option<Vec<u8>>>,

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
        if new.div_ceil(HALF) > old.div_ceil(HALF) {
            self.entities.push(0);
        }

        // Always add the new component states
        for subchunk in self.components.iter_mut().filter_map(|x| x.as_mut()) {
            subchunk.push(0);
        }

        // Update the entity's default state
        self.set_entity_state(self.length - 1, EntityState::Added);
    }

    // State setters
    // Set the entity state of a bundle
    pub fn set_entity_state(&mut self, bundle: usize, state: EntityState) {
        // Write the bit to the chunk
        let chunk = self.entities.get_mut(bundle / HALF).unwrap();
        write_entity_state_u64(chunk, bundle % HALF, state);
    }
    // Set the component state of a bundle using a specific component type
    pub fn set_component_state(&mut self, bundle: usize, mask: Mask, state: bool) {
        // We might need to extend the main vector
        let shift = mask.0.trailing_zeros() as usize;
        if shift >= self.components.len() {
            // Extend the main vector with many empty chunks
            self.components.resize_with(shift + 1, || None);
        }

        // Make sure the current collumn is valid
        let collumn = (&mut self.components[shift]).get_or_insert(vec![0; self.length]);
        let row = collumn.get_mut(bundle).unwrap();

        // Set the bit
        write_bit_u8(row, state, shift % MINI);
    }
    // Set all the components states of a specific component type to "state"
    pub fn set_all_component_states(&mut self, mask: Mask, state: bool) {
        // We are 100% sure that we have a valid collumn
        let shift = mask.0.trailing_zeros() as usize;
        let collumn = self.components[shift].as_mut().unwrap();

        // Set all the states of a specific bit
        for row in collumn {
            write_bit_u8(row, state, shift % MINI);
        }
    }

    // Get the state of an entity
    pub fn get_entity_state(&self, bundle: usize) -> Option<EntityState> {
        // Check if the index is valid
        if bundle >= self.length {
            return None;
        }

        // We are 100% sure that we have a valid collumn
        let state = read_entity_state_u64(*self.entities.get(bundle / HALF).unwrap(), bundle % HALF);
        Some(state)
    }
    // Get the state of a component
    pub fn get_component_state(&self, bundle: usize, mask: Mask) -> Option<bool> {
        // Check if the index is valid
        if bundle >= self.length {
            return None;
        }

        // We are 100% sure that we have a valid collumn
        let shift = mask.0.trailing_zeros() as usize;
        let chunk = self.components[shift].as_ref().unwrap()[bundle];
        Some(read_bit_u8(chunk, shift % MINI))
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

    // Iterators
    pub fn iter_entities(&self) -> EntityStatesIter {
        EntityStatesIter {
            states: self,
            chunk: None,
            bundle: 0,
        }
    }
    /*
    pub fn iter_components(&self) -> ComponentStatesIter {

    }
    */
}

// Entity state iterator
pub struct EntityStatesIter<'a> {
    states: &'a ArchetypeStates,
    chunk: Option<u64>,
    bundle: usize,
}

impl<'a> Iterator for EntityStatesIter<'a> {
    type Item = EntityState;

    fn next(&mut self) -> Option<Self::Item> {
        // Check if the index is valid
        if self.bundle >= self.states.length {
            return None;
        }

        // Invalidate when we reach the end of the chunk
        if self.bundle == HALF - 1 {
            dbg!("reset");
            self.chunk.take();
        }

        // Load a chunk into memory when needed
        let chunk = self.chunk.get_or_insert_with(|| *self.states.entities.get(self.bundle / HALF).unwrap());

        // Super ez
        Some(read_entity_state_u64(*chunk, self.bundle % HALF))
    }
}
