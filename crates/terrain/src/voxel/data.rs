use std::{alloc::Layout, ptr::NonNull};

use crate::MAIN_CHUNK_SIZE;

use super::packed::PackedVoxel;

// Some stored voxel data, in SoA form
pub struct StoredVoxelData {
    densities: NonNull<f32>,
    normals: NonNull<veclib::Vector3<i8>>,
    colors: NonNull<veclib::Vector3<u8>>,
    material_types: NonNull<u8>,
}

impl StoredVoxelData {
    // Allocate enough space to store all the voxels voxels
    pub fn new() -> Self {
        const LEN: usize = (MAIN_CHUNK_SIZE+1).pow(3);
        let densities;
        let normals;
        let colors;
        let material_types;
        unsafe {
            // Allocate the big boy arrays on the heap
            densities = std::alloc::alloc(Layout::array::<f32>(LEN).unwrap()) as *mut f32;
            normals = std::alloc::alloc(Layout::array::<veclib::Vector3<i8>>(LEN).unwrap()) as *mut veclib::Vector3<i8>;
            colors = std::alloc::alloc(Layout::array::<veclib::Vector3<u8>>(LEN).unwrap()) as *mut veclib::Vector3<u8>;
            material_types = std::alloc::alloc(Layout::array::<u8>(LEN).unwrap()) as *mut u8;
        }

        Self {
            densities: NonNull::new(densities).unwrap(),
            normals: NonNull::new(normals).unwrap(),
            colors : NonNull::new(colors).unwrap(),
            material_types : NonNull::new(material_types).unwrap(),
            len,
        }
    }
    // Update the stored voxel data using some packed data that came from the GPU
    pub fn store(&mut self, packed: Vec<PackedVoxel>) -> Self {
        
    }

    // Getters
}

// Drop
impl Drop for StoredVoxelData {
    fn drop(&mut self) {
        // Drop the underlying data
        const LEN: usize = (MAIN_CHUNK_SIZE+1).pow(3);
        unsafe {
            std::alloc::dealloc(self.densities.as_ptr() as *mut u8, Layout::array::<f32>(LEN).unwrap());
            std::alloc::dealloc(self.normals.as_ptr() as *mut u8, Layout::array::<veclib::Vector3<i8>>(LEN).unwrap());
            std::alloc::dealloc(self.colors.as_ptr() as *mut u8, Layout::array::<veclib::Vector3<u8>>(LEN).unwrap());
            std::alloc::dealloc(self.material_types.as_ptr() as *mut u8, Layout::array::<u8>(LEN).unwrap());
        }
    }
}