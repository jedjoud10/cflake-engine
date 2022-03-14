use std::{
    cell::{Ref, RefCell},
    rc::Rc, sync::Arc,
};

use parking_lot::Mutex;

use crate::{unpack_color, PackedVoxelData, CHUNK_SIZE};

// Inner data
struct InnerStoredVoxelData {
    densities: Vec<f32>,
    normals: Vec<vek::Vec3<i8>>,
    colors: Vec<vek::Vec3<u8>>,
    voxel_materials: Vec<u8>,
}

// Some stored voxel data, in SoA form
// This is also clonable because the actual data is stored in an Rc<RefCell<>>
#[derive(Clone)]
pub struct GlobalStoredVoxelData {
    inner: Arc<Mutex<InnerStoredVoxelData>>,
}

impl Default for GlobalStoredVoxelData {
    fn default() -> Self {
        // Allocate enough space to store all the voxels voxels
        const LEN: usize = (CHUNK_SIZE + 1).pow(3);
        let densities = vec![0.0; LEN];
        let normals = vec![vek::Vec3::zero(); LEN];
        let colors = vec![vek::Vec3::zero(); LEN];
        let voxel_materials = vec![0; LEN];

        Self {
            inner: Arc::new(Mutex::new(InnerStoredVoxelData {
                densities,
                normals,
                colors,
                voxel_materials,
            })),
        }
    }
}

impl GlobalStoredVoxelData {
    // Update the stored voxel data using some packed data that came from the GPU
    pub fn store(&mut self, packed: &PackedVoxelData) {
        // We do a bit of overwriting
        let mut inner = self.inner.lock();
        for (i, voxel) in packed.0.iter().enumerate() {
            // Read the voxel attributes
            inner.densities[i] = voxel.density.to_f32();
            inner.colors[i] = unpack_color(voxel.rgb_color);
            inner.normals[i] = voxel.normal;
            inner.voxel_materials[i] = voxel.voxel_material;
        }
    }

    // Getters
    pub fn density(&self, idx: usize) -> f32 {
        *self.inner.lock().densities.get(idx).unwrap()
    }
    pub fn normal(&self, idx: usize) -> vek::Vec3<i8> {
        *self.inner.lock().normals.get(idx).unwrap()
    }
    pub fn color(&self, idx: usize) -> vek::Vec3<u8> {
        *self.inner.lock().colors.get(idx).unwrap()
    }
    pub fn voxel_material(&self, idx: usize) -> u8 {
        *self.inner.lock().voxel_materials.get(idx).unwrap()
    }
}
