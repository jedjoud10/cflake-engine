use super::Voxel;
use super::CHUNK_SIZE;
use super::VoxelGenerator;

// Some chunk data
pub struct ChunkData {
    pub coords: ChunkCoords,
    pub data: Box<[Voxel; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]>,
}

impl Default for ChunkData {
    fn default() -> Self {
        Self {
            coords: ChunkCoords::default(),
            data: Box::new([Voxel::default(); (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]),
        }
    }
}

impl ChunkData {
    // Generate the voxel data needed for mesh construction
    pub fn generate_data(&mut self, voxel_generator: &VoxelGenerator) -> (f32, f32) {
        voxel_generator.generate_data(self.coords.size, self.coords.position, &mut self.data)
    }
}

// The data that will be used to store the position/scale of the chunk
#[derive(Default)]
pub struct ChunkCoords {
    pub position: veclib::Vector3<i64>,
    pub size: u64,
}