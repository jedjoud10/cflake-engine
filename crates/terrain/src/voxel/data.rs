use crate::{PackedVoxelData, CHUNK_SIZE};

// Some stored voxel data, in SoA form
pub struct StoredVoxelData {
    densities: Vec<f32>,
    normals: Vec<veclib::Vector3<i8>>,
    colors: Vec<veclib::Vector3<u8>>,
    voxel_materials: Vec<u8>,
}

impl Default for StoredVoxelData {
    fn default() -> Self {
        // Allocate enough space to store all the voxels voxels
        const LEN: usize = (CHUNK_SIZE + 1).pow(3);
        let densities = vec![0.0; LEN];
        let normals = vec![veclib::Vector3::ZERO; LEN];
        let colors = vec![veclib::Vector3::ZERO; LEN];
        let material_types = vec![0; LEN];

        Self {
            densities,
            normals,
            colors,
            voxel_materials: material_types,
        }
    }
}

// Convert an 8 bit RGB color into a 24 bit RGB color
#[inline(always)]
fn convert_color(packed: u16) -> veclib::Vector3<u8> {
    // 65,535
    let r = (packed >> 11).saturating_mul(8);
    let g = ((packed >> 5) & 63).saturating_mul(4);
    let b = (packed & 31).saturating_mul(8);
    veclib::vec3(r as u8, g as u8, b as u8)
}

impl StoredVoxelData {
    // Update the stored voxel data using some packed data that came from the GPU
    pub fn store(&mut self, packed: &PackedVoxelData) {
        // We do a bit of overwriting
        for (i, voxel) in packed.0.iter().enumerate() {
            // Read the voxel attributes
            self.densities[i] = voxel.density.to_f32();
            self.colors[i] = convert_color(voxel.rgb_color);
            self.normals[i] = voxel.normal;
            self.voxel_materials[i] = voxel.voxel_material;
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
    pub fn voxel_material(&self, idx: usize) -> &u8 {
        self.voxel_materials.get(idx).unwrap()
    }
}
