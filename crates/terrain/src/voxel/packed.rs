use crate::CHUNK_SIZE;

// A packed voxel that has a size of 16 bytes
// This is the final voxel that is returned from the compute shader
#[repr(align(16))]
#[derive(Default, Clone, Copy)]
pub struct PackedVoxel {
    pub density: f32,
    pub normal: veclib::Vector3<i8>,
    padding_: u8,
    pub color: veclib::Vector3<u8>,
    pub material_type: u8,
}

// A vector full of packed voxels
pub struct PackedVoxelData(pub Vec<PackedVoxel>);

impl PackedVoxelData {
    // Create some new packed voxel data that can store enough voxels so that we don't have to reallocate
    pub fn with_voxel_size() -> Self {
        const CAP: usize = (CHUNK_SIZE + 1).pow(3);
        Self(vec![PackedVoxel::default(); CAP])
    }
}
