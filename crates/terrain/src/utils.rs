use crate::{MAIN_CHUNK_SIZE};

// Casually stole my old code lol
// Get the position from an index
pub fn unflatten(mut index: usize) -> (usize, usize, usize) {
    let z = index / (MAIN_CHUNK_SIZE+1);
    index -= z * (MAIN_CHUNK_SIZE+1);
    let y = index / ((MAIN_CHUNK_SIZE+1) * (MAIN_CHUNK_SIZE+1));
    let x = index % (MAIN_CHUNK_SIZE+1);
    return (x, y, z);
}
// Get the index from a position
pub fn flatten(position: (usize, usize, usize)) -> usize {
    return position.0 + (position.1 * (MAIN_CHUNK_SIZE+1) * (MAIN_CHUNK_SIZE+1)) + (position.2 * (MAIN_CHUNK_SIZE+1));
}