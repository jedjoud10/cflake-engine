use std::ops::{Index, IndexMut};
use half::f16;

// A simple voxel that has a size of 16 bytes
// This is the final voxel that is returned from the compute shader
#[repr(C, align(16))]
#[derive(Clone)]
pub struct Voxel {
    // The density of the voxel stored in a 16 bit half float
    pub density: f16,
    // The normals stored in a vec3 full of f16s
    pub normal: veclib::Vector3<f16>,
    // 8 Bytes left

    // The color of each voxel is also stored in vec3 of u8s
    pub color: veclib::Vector3<u8>,
    // Also store the hardness of the voxel
    pub hardness: u8,
    // 4 Bytes left

    // Material type
    pub material_type: u8,
    pub _test: [u8; 3],
}

// Some voxel data. Thiis contains the whole voxels array, that is actually stored on the heap
pub struct VoxelData(pub Box<[Voxel]>);

impl Index<usize> for VoxelData {
    type Output = Voxel;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.get(index).unwrap()
    }
}

impl IndexMut<usize> for VoxelData {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.get_mut(index).unwrap()
    }
}
