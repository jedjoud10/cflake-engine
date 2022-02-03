use std::{alloc::Layout, ptr::NonNull};

use crate::{PackedVoxelData, CHUNK_SIZE};

use super::packed::PackedVoxel;

// Some stored voxel data, in SoA form
pub struct StoredVoxelData {
    densities: Vec<f32>,
    normals: Vec<veclib::Vector3<i8>>,
    colors: Vec<veclib::Vector3<u8>>,
    material_types: Vec<u8>,
}

impl StoredVoxelData {
    // Allocate enough space to store all the voxels voxels
    pub fn new() -> Self {
        const LEN: usize = (CHUNK_SIZE + 1).pow(3);
        let densities = vec![0.0; LEN];
        let normals = vec![veclib::Vector3::ZERO; LEN];
        let colors = vec![veclib::Vector3::ZERO; LEN];
        let material_types = vec![0; LEN];

        Self {
            densities,
            normals,
            colors,
            material_types,
        }
    }
    // Update the stored voxel data using some packed data that came from the GPU
    pub fn store(&mut self, packed: &PackedVoxelData) {
        // We do a bit of overwriting
        for (i, voxel) in packed.0.iter().enumerate() {
            // Read the voxel attributes
            self.densities[i] = voxel.density;
            self.normals[i] = voxel.normal;
            self.colors[i] = voxel.color;
            self.material_types[i] = voxel.material_type;
        }
    }

    // Getters
    pub fn density(&self, idx: usize) -> &f32 {
        self.densities.get(idx).unwrap()
    }
    pub fn normal(&self, idx: usize) -> &veclib::Vector3<i8> {
        self.normals.get(idx).unwrap()
    }
    pub fn color(&self, idx: usize) -> &veclib::Vector3<u8> {
        self.colors.get(idx).unwrap()
    }
    pub fn material_type(&self, idx: usize) -> &u8 {
        self.material_types.get(idx).unwrap()
    }
}
