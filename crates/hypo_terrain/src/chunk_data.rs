use super::Voxel;
use super::CHUNK_SIZE;
use super::VoxelGenerator;

// Some chunk data
pub struct ChunkData {
    pub position: veclib::Vector3<i64>,
    pub size: u64,
    pub data: Box<[Voxel; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]>,
}

impl Default for ChunkData {
    fn default() -> Self {
        Self {
            position: veclib::Vector3::<i64>::default_zero(),
            size: 0,
            data: Box::new([Voxel::default(); (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]),
        }
    }
}

impl ChunkData {
    // Generate the voxel data needed for mesh construction
    pub fn generate_data(&mut self, voxel_generator: &VoxelGenerator) -> (f32, f32) {
        voxel_generator.generate_data(self.size, self.position, &mut self.data)
    }
}