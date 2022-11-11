use crate::Mask;

// A single chunk that will be contained within the archetype component column
struct StateColumnChunk {
    added: u64,
    removed: u64,
    modified: u64,
}

// Component state chunk that contains the component states for a bundle
// TODO: Description
#[derive(Clone, Copy, Debug)]
pub struct StateRow(Mask, Mask, Mask);

impl StateRow {
    // Create a new state row with raw values
    pub fn new(added: Mask, removed: Mask, mutated: Mask) -> Self {
        Self(added, removed, mutated)
    }

    // Get the added state mask
    pub fn added(&self) -> Mask {
        self.0
    }

    // Get the removed mask
    pub fn removed(&self) -> Mask {
        self.1
    }

    // Get the mutated state mask
    pub fn mutated(&self) -> Mask {
        self.2
    }

    // Execute a callback that will modify the masks, and return their old values
    pub fn update(&mut self, f: impl FnOnce(&mut Mask, &mut Mask, &mut Mask)) -> StateRow {
        let old = *self;
        f(&mut self.0, &mut self.1, &mut self.2);
        old
    }
}
