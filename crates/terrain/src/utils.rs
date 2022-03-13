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
pub fn flatten_vec3(position: vek::Vec3<usize>) -> usize {
    position.x + (position.y * (CHUNK_SIZE + 1) * (CHUNK_SIZE + 1)) + (position.z * (CHUNK_SIZE + 1))
}

// Convert a 16 bit RGB color into a 24 bit RGB color
pub fn unpack_color(packed: u16) -> vek::Vec3<u8> {
    // 65,535
    let r = (packed >> 11).saturating_mul(8);
    let g = ((packed >> 5) & 63).saturating_mul(4);
    let b = (packed & 31).saturating_mul(8);
    vek::Vec3::new(r as u8, g as u8, b as u8)
}

// Convert an 24 bit RGB color into a 16 bit RGB color
pub fn pack_color(unpacked: vek::Vec3<u8>) -> u16 {
    // 65,535
    let r = ((unpacked.x / 8) as u16) << 11;
    let g = ((unpacked.y / 4) as u16) << 5;
    let b = (unpacked.z / 8) as u16;
    r | g | b
}
