use half::f16;
use std::ops::{Index, IndexMut};

// A simple voxel that has a size of 16 bytes
// This is the final voxel that is returned from the compute shader
#[repr(align(16))]
pub struct Voxel {
    // The density of the voxel stored in a 16 bit half float
    pub density: f16,
    // The normals stored in a vec3 full of f16s
    pub normal: veclib::Vector3<f16>,
    // 8 Bytes left

    // The color of each voxel is also stored in vec3 of u8s
    pub color: veclib::Vector3<u8>,
    // 5 Bytes left

    // Material type
    pub material_type: u8,
    pub _test: [u8; 4],
}

// Some details about a valid voxel generation
pub struct ValidGeneratedVoxelData {
    pub voxels: Vec<Voxel>,
    pub valid_sub_regions: u8,
}

// Some info about some generated voxel data
#[derive(Default)]
pub struct GeneratedVoxelData {
    pub data: Option<ValidGeneratedVoxelData>,
    pub generated: bool,    
}