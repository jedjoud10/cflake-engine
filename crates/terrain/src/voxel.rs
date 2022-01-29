use std::ops::{Index, IndexMut};

use rendering::basics::model::Model;

// A simple voxel wrapper that contains some default voxel values and some extra values
#[repr(C)]
pub struct VoxelWrapper<T: Voxable> {
    // Default values
    pub density: f32,
    
    // Voxable values
    pub normal: veclib::Vector4<f32>,
    pub extra: T,
}


// A voxel trait that can contain some extra data
pub trait Voxable
    where Self: Sized
{
    // Interpolate between two voxels values
    fn interpolate(v1: &Self, v2: &Self, t: f32) -> Self;
    // Add our extra values to a model (ex: custom model data like tint)
    fn push(self, model: &mut Model) {}
}

// Some voxel data. Thiis contains the whole voxels array, that is actually stored on the heap
pub struct VoxelData<V: Voxable>(pub Box<[VoxelWrapper<V>]>);

impl<U: Voxable> Index<usize> for VoxelData<U> {
    type Output = VoxelWrapper<U>;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.get(index).unwrap()
    }
}

impl<U: Voxable> IndexMut<usize> for VoxelData<U> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0.get_mut(index).unwrap()
    }
}
