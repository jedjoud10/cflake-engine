use std::ops::{Index, IndexMut};

// A simple voxel that has a size of 16 bytes
// This is the final voxel that is returned from the compute shader
#[repr(align(16))]
pub struct Voxel {
    pub density: f32,
    pub normal: veclib::Vector3<i8>,
    padding_: u8,
    pub color: veclib::Vector3<u8>,
    pub material_type: u8,
}

// Some details about a valid voxel generation
pub struct ValidGeneratedVoxelData {
    pub voxels: Vec<Voxel>,
    pub valid_sub_regions: u16,
}

// Some info about some generated voxel data
#[derive(Default)]
pub struct GeneratedVoxelData {
    pub data: Option<ValidGeneratedVoxelData>,
    pub generated: bool,    
}