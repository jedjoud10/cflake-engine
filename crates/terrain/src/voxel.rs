use std::ops::{Index, IndexMut};

// A voxel that we store temporarily
#[derive(Default, Clone, Copy)]
pub struct TempVoxel {
    pub density: u16,
    pub normal: veclib::Vector3<u8>,
}
// Some voxel data. Thiis contains the whole voxels array, that is actually stored on the heap
pub struct TempVoxelData(pub Box<[TempVoxel]>);

impl Index<usize> for TempVoxelData {
    type Output = TempVoxel;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.get(index).unwrap()
    }
}

impl IndexMut<usize> for TempVoxelData {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.get_mut(index).unwrap()
    }
}
