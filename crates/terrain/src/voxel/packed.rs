use half::f16;

use crate::CHUNK_SIZE;

// A packed voxel that has a size of 16 bytes
// This is the final voxel that is returned from the compute shader
#[repr(align(8))]
#[derive(Default, Clone, Copy)]
pub struct PackedVoxel {
    pub density: f16,                // 2
    pub rgb_color: u16,              // 2
    pub normal: vek::Vec3<i8>, // 3
    pub voxel_material: u8,          // 1
}

// A vector full of packed voxels
pub struct PackedVoxelData(pub Vec<PackedVoxel>);

impl Default for PackedVoxelData {
    fn default() -> Self {
        const CAP: usize = (CHUNK_SIZE + 1).pow(3);
        Self(vec![PackedVoxel::default(); CAP])
    }
}
