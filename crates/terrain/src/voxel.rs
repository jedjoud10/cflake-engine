// Just a simple voxel
#[derive(Default, Clone, Copy)]
pub struct Voxel {
    pub density: f32,
    pub normal: veclib::Vector3<f32>,
    pub material_id: u8, // Voxel material
}
// Some voxel data. Thiis contains the whole voxels array, that is actually stored on the heap
pub struct VoxelData {
    pub voxels: Box<[Voxel]>
}

