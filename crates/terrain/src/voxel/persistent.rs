use bitfield::SparseBitfield;
// Persistent voxel data that is stored for each individual chunk
pub struct PersistentVoxelData {
    // Solid state for each voxel
    pub solid: SparseBitfield,

    // Material type for each voxel
    pub voxel_materials: Vec<u8>,
}

impl PersistentVoxelData {
    // Check if a voxel is solid or not
    pub fn is_solid(&self, i: usize) -> bool {
        self.solid.get(i)
    }
    // Get the voxel material at a s pecific index
    pub fn voxel_material(&self, idx: usize) -> u8 {
        *self.voxel_materials.get(idx).unwrap()
    }
}
