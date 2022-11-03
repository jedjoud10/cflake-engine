use crate::HiBitSet;

// A hopmap is a collection that stores multiple indices to allow us to iterate through collections much faster
// A hopmap will literally discard all the iteration between certain ranges and "hop" to a specific index when needed
// The indices stored in the hopmap must be sorted
pub struct HopMap {
    current: Vec<usize>,
    target: Vec<usize>,
}

impl HopMap {
    // Create a hop map from two vectors
    pub fn from_vectors(mut current: Vec<usize>, mut target: Vec<usize>) -> Self {
        current.sort();
        target.sort();

        Self { current, target }
    }

    // Create an empty hop map (placeholder)
    pub fn new() -> Self {
        Self {
            current: Vec::new(),
            target: Vec::new(),
        }
    }

    // Given an index, get the next hop index it hop at (if possible)
    pub fn hop_to(&self, index: usize) -> Option<usize> {
        self.current
            .iter()
            .position(|i| index > *i)
            .map(|index| self.target[index])
    }
}

impl From<HiBitSet> for HopMap {
    fn from(value: HiBitSet) -> Self {
        todo!()
    }
}
