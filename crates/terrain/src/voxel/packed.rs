// A packed voxel that has a size of 16 bytes
// This is the final voxel that is returned from the compute shader
#[repr(align(16))]
pub struct PackedVoxel {
    pub density: f32,
    pub normal: veclib::Vector3<i8>,
    padding_: u8,
    pub color: veclib::Vector3<u8>,
    pub material_type: u8,
}