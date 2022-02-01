use crate::MAIN_CHUNK_SIZE;

// Casually stole my old code lol
// Get the position from an index
pub fn unflatten(mut index: usize) -> (usize, usize, usize) {
    let z = index / (MAIN_CHUNK_SIZE + 1);
    index -= z * (MAIN_CHUNK_SIZE + 1);
    let y = index / ((MAIN_CHUNK_SIZE + 1) * (MAIN_CHUNK_SIZE + 1));
    let x = index % (MAIN_CHUNK_SIZE + 1);
    (x, y, z)
}
// Get the index from a position
pub fn flatten(position: (usize, usize, usize)) -> usize {
    position.0 + (position.1 * (MAIN_CHUNK_SIZE + 1) * (MAIN_CHUNK_SIZE + 1)) + (position.2 * (MAIN_CHUNK_SIZE + 1))
}
// Get the index from a position
pub fn flatten_custom(position: (usize, usize, usize), size: usize) -> usize {
    position.0 + ((position.1 * size) * size) + (position.2 * size)
}
