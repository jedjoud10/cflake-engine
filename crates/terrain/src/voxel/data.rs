use crate::{unpack_color, PackedVoxelData, PersistentVoxelData, VoxelState, VoxelStateSet, CHUNK_SIZE};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};

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
    pub fn store(&mut self, packed: &PackedVoxelData) -> PersistentVoxelData {
        let voxels = &packed.0;
        let densities = &mut self.densities;
        let colors = &mut self.colors;
        let normals = &mut self.normals;
        let voxel_materials = &mut self.voxel_materials;

        // Get the combined voxel states
        let states = voxels
            .par_iter()
            .zip(densities)
            .zip(colors)
            .zip(normals)
            .zip(voxel_materials)
            .map(|((((voxel, density), color), normal), voxel_material)| {
                // Read the voxel attributes while we're at it
                *density = voxel.density.to_f32();
                *color = unpack_color(voxel.rgb_color);
                *normal = voxel.normal;
                *voxel_material = voxel.voxel_material;
            });

        // TODO: Fix

        PersistentVoxelData {}
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

    // Iterators
    pub fn iter_densities(&self) -> impl Iterator<Item = &f32> {
        self.densities.iter()
    }
    pub fn iter_normals(&self) -> impl Iterator<Item = &vek::Vec3<i8>> {
        self.normals.iter()
    }
    pub fn iter_colors(&self) -> impl Iterator<Item = &vek::Rgb<u8>> {
        self.colors.iter()
    }
    pub fn iter_voxel_materials(&self) -> impl Iterator<Item = &u8> {
        self.voxel_materials.iter()
    }
}
