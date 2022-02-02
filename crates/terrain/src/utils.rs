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
// Get the index from a position
pub fn flatten_custom(position: veclib::Vector3<usize>, size: usize) -> usize {
    position.x + (position.y * size) + (position.z * size * size)
}
// Inverse of lerp
fn inverse_lerp(a: f32, b: f32, x: f32) -> f32 {
    (x - a) / (b - a)
}
