



use crate::{unpack_color, PackedVoxelData, CHUNK_SIZE};

// Some stored voxel data, in SoA form
pub struct VoxelData {
    densities: Vec<f32>,
    normals: Vec<vek::Vec3<i8>>,
    colors: Vec<vek::Rgb<u8>>,
    voxel_materials: Vec<u8>,
}

impl Default for VoxelData {
    fn default() -> Self {
        // Allocate enough space to store all the voxels voxels
        const LEN: usize = (CHUNK_SIZE + 1).pow(3);
        let densities = vec![0.0; LEN];
        let normals = vec![vek::Vec3::zero(); LEN];
        let colors = vec![vek::Rgb::zero(); LEN];
        let voxel_materials = vec![0; LEN];

        Self {
            densities,
            normals,
            colors,
            voxel_materials,
        }
    }
}

impl VoxelData {
    // Update the stored voxel data using some packed data that came from the GPU
    pub fn store(&mut self, packed: &PackedVoxelData) {
        // We do a bit of overwriting
        for (i, voxel) in packed.0.iter().enumerate() {
            // Read the voxel attributes
            self.densities[i] = voxel.density.to_f32();
            self.colors[i] = unpack_color(voxel.rgb_color);
            self.normals[i] = voxel.normal;
            self.voxel_materials[i] = voxel.voxel_material;
        }
    }

    // Getters
    pub fn density(&self, idx: usize) -> f32 {
        *self.densities.get(idx).unwrap()
    }
    pub fn normal(&self, idx: usize) -> vek::Vec3<i8> {
        *self.normals.get(idx).unwrap()
    }
    pub fn color(&self, idx: usize) -> vek::Rgb<u8> {
        *self.colors.get(idx).unwrap()
    }
    pub fn voxel_material(&self, idx: usize) -> u8 {
        *self.voxel_materials.get(idx).unwrap()
    }
}
