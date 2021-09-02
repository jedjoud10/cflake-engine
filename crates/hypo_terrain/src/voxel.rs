use super::terrain::Terrain;
use super::CHUNK_SIZE;

// Casually stole my old code lol
// Get the position from an index
pub fn unflatten(mut index: usize) -> (usize, usize, usize) {
    let z = index / (CHUNK_SIZE);
    index -= z * (CHUNK_SIZE);
    let y = index / (CHUNK_SIZE * CHUNK_SIZE);
    let x = index % (CHUNK_SIZE);
    return (x, y, z);
}
// Get the index from a position
pub fn flatten(position: (usize, usize, usize)) -> usize {
    return position.0 + (position.1 * CHUNK_SIZE * CHUNK_SIZE) + (position.2 * CHUNK_SIZE);
}

// Handles the generation of voxel data
#[derive(Default)]
pub struct VoxelGenerator {}

// Just a simple voxel
#[derive(Default, Clone, Copy)]
pub struct Voxel {
    pub density: f32,
}

impl VoxelGenerator {
    // Set the default values
    pub fn set_values(&mut self, terrain: &Terrain) {}
    // Get the voxel at a specific point
    pub fn get_voxel(&self, point: veclib::Vector3<f32>) -> Voxel {
        let mut voxel: Voxel = Voxel { density: 0.0 };
        // Code goes here
        voxel.density = point.y() - 40.0;
        voxel.density += (point.x() * 0.05).sin() * 10.0;
        voxel.density += (point.z() * 0.05).sin() * 3.0;
        voxel.density = point.y() - 5.0 + (point.z() * 0.2).sin() * 20.0;
        //voxel.density = point.y() - 5.0;
        return voxel;
    }
}
