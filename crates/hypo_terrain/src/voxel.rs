use hypo_rendering::{Shader, Texture3D};
use hypo_system_event_data::SystemEventData;

use super::terrain::Terrain;
use super::CHUNK_SIZE;

// Casually stole my old code lol
// Get the position from an index
pub fn unflatten(mut index: usize) -> (usize, usize, usize) {
    let z = index / (CHUNK_SIZE);
    index -= z * (CHUNK_SIZE);
    let y = index / (CHUNK_SIZE * CHUNK_SIZE);
    let x = index % (CHUNK_SIZE);
    return (x, y, z);
}
// Get the index from a position
pub fn flatten(position: (usize, usize, usize)) -> usize {
    return position.0 + (position.1 * CHUNK_SIZE * CHUNK_SIZE) + (position.2 * CHUNK_SIZE);
}

// Handles the generation of voxel data
#[derive(Default)]
pub struct VoxelGenerator {
    // The compute shader's name used for voxel generation
    pub compute_shader_name: String,
    // The 3D texture used for voxel generation as well
    pub voxel_texture: Texture3D,
}

impl VoxelGenerator {
    // Generate the voxels using a compute shader
    pub fn generate_voxels(&self, event_data: &SystemEventData, size: u64, position: veclib::Vector3<i64>, data: &mut Box<[Voxel; (CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as usize]>) -> Option<()> {
        // Get the compute shader
        let compute = event_data.shader_cacher.1.get_object(self.compute_shader_name.as_str()).unwrap();
        
        // Set the compute shader variables and voxel texture
        compute.set_i3d("voxel_image", &self.voxel_texture, hypo_rendering::TextureShaderAccessType::ReadWrite);

        // Run the compute shader
        compute.run_compute((CHUNK_SIZE as u32, CHUNK_SIZE as u32, CHUNK_SIZE as u32));

        // Read back the texture into the data buffer
        let pixels = self.voxel_texture.internal_texture.fill_array::<veclib::Vector3<f32>, f32>();
        

        return None;
        return Some(());
    }
}

// Just a simple voxel
#[derive(Default, Clone, Copy)]
pub struct Voxel {
    pub density: f32,
}