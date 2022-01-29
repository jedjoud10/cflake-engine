use std::ops::{Index, IndexMut};

// A voxel trait that can contain some final voxel data that we read from the compute shader
pub trait Voxable {
    // Interpolate between two voxels
    fn interpolate(v1: Self, v2: Self) -> Self;
    // Get the density of our voxable
    fn density() -> f32 {  }
    // Get the normals of our voxable
}

// Some voxel data. Thiis contains the whole voxels array, that is actually stored on the heap
pub struct VoxelData<V: Voxable>(pub Box<[V]>);

impl<U: Voxable> Index<usize> for VoxelData<U> {
    type Output = U;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.get(index).unwrap()
    }
}

impl<U: Voxable> IndexMut<usize> for VoxelData<U> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.get_mut(index).unwrap()
    }
}
