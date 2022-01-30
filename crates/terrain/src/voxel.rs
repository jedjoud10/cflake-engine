use std::ops::{Index, IndexMut};

use rendering::basics::model::Model;

// A voxel trait that can contain some extra data
pub trait Voxable
where
    Self: Sized,
{
    // Interpolate between two voxels values
    fn interpolate(v1: &Self, v2: &Self, t: f32) -> Self;
    // Add our extra values to a model (ex: custom model data like tint)
    fn push(self, _model: &mut Model) {}
    // Get the density of this voxel
    fn density(&self) -> f32;
    // Get the normal of this voxel
    fn normal(&self) -> veclib::Vector3<f32>;
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
