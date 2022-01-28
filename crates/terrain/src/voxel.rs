use std::ops::{Index, IndexMut};

// A voxel that we store temporarily
#[derive(Default, Clone, Copy)]
pub struct Voxel {
    pub density: f32,
    pub normal: veclib::Vector3<f32>,
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
