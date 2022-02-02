use crate::CHUNK_SIZE;

// Casually stole my old code lol
// Get the position from an index
pub fn unflatten(mut index: usize) -> (usize, usize, usize) {
    let z = index / (CHUNK_SIZE + 1);
    index -= z * (CHUNK_SIZE + 1);
    let y = index / ((CHUNK_SIZE + 1) * (CHUNK_SIZE + 1));
    let x = index % (CHUNK_SIZE + 1);
    (x, y, z)
}
// Get the index from a position
pub fn flatten(position: (usize, usize, usize)) -> usize {
    position.0 + (position.1 * (CHUNK_SIZE + 1) * (CHUNK_SIZE + 1)) + (position.2 * (CHUNK_SIZE + 1))
}
pub fn flatten_vec3(position: veclib::Vector3<usize>) -> usize {
    position.x + (position.y * (CHUNK_SIZE + 1) * (CHUNK_SIZE + 1)) + (position.z * (CHUNK_SIZE + 1))
}