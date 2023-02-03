// Number of bits in a usize as an usize
const BITS: usize = usize::BITS as usize;

// A single chunk that will be contained within the archetype component column
#[derive(Default, Clone, Copy)]
pub struct StateColumnChunk {
    // Keeps track of added components
    pub added: usize,

    // Keeps track of mutated (modified) components
    pub modified: usize,
}

// Returned from the Vec<StateColumnChunk>
#[derive(Default, Clone, Copy)]
pub struct StateFlags {
    pub added: bool,
    pub modified: bool,
}

// A single column of archetype entity states
#[derive(Default)]
pub struct StateColumn(Vec<StateColumnChunk>, usize);

// Update a value in a specific bitmask, though return the unwritten value first
fn toggle_bit(
    bitmask: &mut usize,
    index: usize,
    value: bool,
) -> bool {
    let copy = (*bitmask >> index) & 1 == 1;

    if value {
        *bitmask |= 1 << index;
    } else {
        *bitmask &= !(1 << index);
    }

    copy
}

// Enable all the bits between "start" and "end" in the binary representation of a usize
// Start is inclusive, end is exclusive
pub(crate) fn enable_in_range(start: usize, end: usize) -> usize {
    assert!(end >= start);

    if end == BITS {
        !((1usize << (start)) - 1usize)
    } else if start == BITS {
        0
    } else {
        ((1usize << (start)) - 1usize) ^ ((1usize << end) - 1usize)
    }
}

// Check if a bit at a specific index is set
fn is_bit_enabled(bitset: usize, index: usize) -> bool {
    bitset >> index & 1 == 1
}

impl StateColumn {
    // Add new n number of entries that all contain the same state flags
    // This requires the old_len and new_len calculated within the extend_from_slice method inside the Archetype
    pub(crate) fn extend_with_flags(
        &mut self,
        additional: usize,
        flags: StateFlags,
    ) {
        // Make sure the states have enough chunks to deal with
        let old_len = self.1;
        let new_len = self.1 + additional;
        let new_len_chunks = new_len / BITS;
        let iter = std::iter::repeat(StateColumnChunk::default());
        let iter = iter.take((new_len_chunks + 1) - self.0.len());
        self.0.extend(iter);
        self.1 = new_len;

        // Convert the flags into masks
        let added = flags.added as usize * usize::MAX;
        let modified = flags.modified as usize * usize::MAX;

        // Update the chunk bits
        for (i, chunk) in self.0.iter_mut().enumerate() {
            let start = i * BITS;
            let end = (i + 1) * BITS;

            // Skip this chunk if it won't be modified
            if old_len > end || new_len < start {
                continue;
            }

            // Create start and end ranges that will be clamped to old_len and new_len respectively
            let local_start =
                usize::saturating_sub(old_len, start).min(BITS - 1);
            let local_end =
                usize::saturating_sub(new_len, start).min(BITS);

            // Bit magic that will enable all the bits between local_start and local_end;
            let range = enable_in_range(local_start, local_end);
            chunk.added |= range & added;
            chunk.modified |= range & modified;
        }
    }

    // Reserve a specific amount of entries within the state column
    pub(crate) fn reserve(&mut self, additional: usize) {
        let current = self.0.len();
        let new = ((self.1 + additional) as f32 / BITS as f32).ceil()
            as usize;
        assert!(new >= current);
        self.0.reserve(new - current);
    }

    // Shrink the memory allocation so it takes less space
    pub(crate) fn shrink_to_fit(&mut self) {
        self.0.shrink_to_fit();
    }

    // Remove a specific element and replace it's current location with the last element
    pub(crate) fn swap_remove(
        &mut self,
        index: usize,
    ) -> Option<StateFlags> {
        // Cannot remove non-existant index
        if index >= self.1 {
            return None;
        }

        // Fetch the values of the last element
        let last = self.0.last().map(|chunk| {
            let last_local_index = self.1 % BITS;
            StateFlags {
                added: is_bit_enabled(chunk.added, last_local_index),
                modified: is_bit_enabled(
                    chunk.modified,
                    last_local_index,
                ),
            }
        });

        // Replace the entry at "index" with "last"
        last.map(|flags| {
            let chunk = index / BITS;
            let local = index % BITS;
            let chunk = &mut self.0[chunk];
            self.1 -= 1;

            // Decompose the state flags
            let StateFlags { added, modified } = flags;

            let added = toggle_bit(&mut chunk.added, local, added);
            let modified =
                toggle_bit(&mut chunk.modified, local, modified);
            StateFlags { added, modified }
        })
    }

    // Remove a specific element and replace it's current location with the last element
    // This will also insert the removed element as a new entry into another state column
    pub(crate) fn swap_remove_move(
        &mut self,
        index: usize,
        other: &mut Self,
    ) {
        let removed = self.swap_remove(index);

        if let Some(removed) = removed {
            other.extend_with_flags(1, removed);
        }
    }

    // Update a specific entry using a callback and it's index
    pub(crate) fn update(
        &mut self,
        index: usize,
        update: impl FnOnce(&mut StateFlags),
    ) {
        let chunk = index / BITS;
        let location = index % BITS;
        let chunk = &mut self.0[chunk];
        let mut flags = StateFlags {
            added: is_bit_enabled(chunk.added, location),
            modified: is_bit_enabled(chunk.modified, location),
        };
        update(&mut flags);

        toggle_bit(&mut chunk.added, location, flags.added);
        toggle_bit(&mut chunk.modified, location, flags.modified);
    }

    // Get an immutable slice over all the chunks
    pub fn chunks(&self) -> &[StateColumnChunk] {
        &self.0
    }

    // Get a mutable slice over all the chunks
    pub(crate) fn chunks_mut(&mut self) -> &mut [StateColumnChunk] {
        &mut self.0
    }

    // Get a specific state column chunk immutably
    pub fn get_chunk(
        &self,
        index: usize,
    ) -> Option<&StateColumnChunk> {
        self.0.get(index)
    }

    // Get a specific state column entry immutably
    pub fn get(&self, index: usize) -> Option<StateFlags> {
        let chunk = index / BITS;
        let location = index % BITS;
        let chunk = &mut self.0.get(chunk)?;
        let flags = StateFlags {
            added: is_bit_enabled(chunk.added, location),
            modified: is_bit_enabled(chunk.modified, location),
        };
        Some(flags)
    }

    // Get a specific state column chunk mutably
    pub(crate) fn get_mut_chunk(
        &mut self,
        index: usize,
    ) -> Option<&mut StateColumnChunk> {
        self.0.get_mut(index)
    }

    // Get the number of component states we have
    pub fn len(&self) -> usize {
        self.1
    }

    // Clear all the states from within this column
    pub(crate) fn reset(&mut self) {
        for chunk in self.0.iter_mut() {
            chunk.added = 0usize;
            chunk.modified = 0usize;
        }
    }

    // Clear all the states from within this column
    pub(crate) fn clear(&mut self) {
        self.0.clear();
        self.1 = 0;
    }
}
