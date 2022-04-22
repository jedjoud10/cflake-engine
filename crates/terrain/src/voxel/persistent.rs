// Persistent voxel data that is stored for each individual chunk
pub struct PersistentVoxelData {
    // Stores the packed voxel data
}

impl PersistentVoxelData {
    // Check if a voxel is solid or not
    pub fn is_solid(&self, i: usize) -> bool {
        //self.solid.get(i)
        false
    }
    // Get the voxel material at a s pecific index
    pub fn voxel_material(&self, idx: usize) -> u8 {
        0
    }
}
