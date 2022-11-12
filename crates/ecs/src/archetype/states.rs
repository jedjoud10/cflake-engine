use crate::Mask;

// A single chunk that will be contained within the archetype component column
#[derive(Default, Clone, Copy)]
pub struct StateColumnChunk {
    added: u64,
    removed: u64,
    modified: u64,
}

// Returned from the Vec<StateColumnChunk>
#[derive(Default, Clone, Copy)]
pub struct StateFlags {
    pub added: bool,
    pub modified: bool,
    pub removed: bool,
}

// A single column of archetype entity states
#[derive(Default)]
pub struct StateColumn(Vec<StateColumnChunk>, usize);

// Update a value in a specific bitmask, though return the unwritten value first
fn set_bit(bitmask: &mut u64, index: usize, value: bool) -> bool {
    let copy = (*bitmask >> index) & 1 == 1;

    if value {
        *bitmask |= 1 << index;
    } else {
        *bitmask &= !(1 << index);
    }

    return copy;
}

impl StateColumn {
    // Add new n number of entries that all contain the same state flags
    // This requires the old_len and new_len calculated within the extend_from_slice method inside the Archetype
    pub(crate) fn extend_with_flags(&mut self, additional: usize, flags: StateFlags) {
        // Make sure the states have enough chunks to deal with
        let old_len = self.1;
        let new_len = self.1 + additional;
        let iter = std::iter::repeat(StateColumnChunk::default());
        let iter = iter.take(additional / u64::BITS as usize);
        self.0.extend(iter);

        // Update the chunk bits
        for (i, chunk) in self.0.iter_mut().enumerate() {
            let start = i * u64::BITS as usize;
            let local_start = usize::saturating_sub(old_len, start).min(u64::BITS as usize);
            let local_end = usize::saturating_sub(new_len, start).min(u64::BITS as usize);

            // Bit magic that will enable all the bits between local_start and local_end;
            let range = ((1u64 << (local_start + 1)) - 1u64) ^ ((1u64 << local_end) - 1u64);
            chunk.added |= range & (flags.added as u64);
            chunk.modified |= range & (flags.modified as u64);
            chunk.removed |= range & (flags.removed as u64);
        }
    } 

    // Reserve a specific amount of entries within the state column
    pub(crate) fn reserve(&mut self, additional: usize) {
        let current = self.0.capacity() * u64::BITS as usize;
        let new = self.0.capacity() + u64::BITS as usize + additional;
        let current_num_chunks = (current as f32 / u64::BITS as f32).ceil() as usize;
        let new_num_chunks = (new as f32 / u64::BITS as f32).ceil() as usize;
        let additional_chunks =  new_num_chunks - current_num_chunks;
        self.0.reserve(additional_chunks);
    }

    // Remove a specific element and replace it's current location with the last element
    pub(crate) fn swap_remove(&mut self, index: usize) -> Option<StateFlags> {
        // Fetch the values of the last element
        let last = self.0.last().map(|chunk| {
          StateFlags {
            added: chunk.added >> (self.1-1) & 1 == 1,
            removed: chunk.removed >> (self.1-1) & 1 == 1,
            modified: chunk.modified >> (self.1-1) & 1 == 1,
          }
        });

        // Replace the entry at "index" with "last"
        last.map(|flags| {
            let chunk = index / u64::BITS as usize;
            let local = index % u64::BITS as usize;
            let chunk = &mut self.0[chunk];
            self.1 -= 1;

            let added = set_bit(&mut chunk.added, local, flags.added);
            let modified = set_bit(&mut chunk.modified, local, flags.modified);
            let removed = set_bit(&mut chunk.removed, local, flags.removed);        
            StateFlags { added, removed, modified }
        })
    }

    // Remvoe a speciifc element and replace it's current location with the last element
    // This will also insert the removed element as a new entry into another state column
    pub(crate) fn swap_remove_move(&mut self, index: usize, other: &mut Self) {
        let removed = self.swap_remove(index);

        if let Some(removed) = removed {
            other.extend_with_flags(1, removed);
        }
    }

    // Update a specific entry using a callback
    pub(crate) fn update(&mut self, index: usize, update: impl FnOnce(&mut StateFlags)) {
        let chunk = index / u64::BITS as usize;
        let location = index % u64::BITS as usize;
        let chunk = &mut self.0[chunk];
        let mut flags = StateFlags {
            added: chunk.added >> (self.1-1) & 1 == 1,
            modified: chunk.modified >> (self.1-1) & 1 == 1,
            removed: chunk.removed >> (self.1-1) & 1 == 1,
        };
        update(&mut flags);

        set_bit(&mut chunk.added, index, flags.added);
        set_bit(&mut chunk.modified, index, flags.modified);
        set_bit(&mut chunk.removed, index, flags.removed);
    }

    // Clear all the states from within this column
    pub(crate) fn clear(&mut self) {
        for chunk in self.0.iter_mut() {
            chunk.added = 0u64;
            chunk.modified = 0u64;
            chunk.removed = 0u64;
        }
    }
}