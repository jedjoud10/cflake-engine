use rendering::{Shader, Texture2D, Texture3D};
use system_event_data::SystemEventData;

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
    // Generate the voxel texture
    pub fn setup_voxel_generator(&mut self, event_data: &mut SystemEventData) {
        // Create the voxel texture
        self.voxel_texture = Texture3D::new()
            .set_dimensions(CHUNK_SIZE as u16, CHUNK_SIZE as u16, CHUNK_SIZE as u16)
            .set_idf(gl::R32F, gl::RED, gl::FLOAT)
            .set_wrapping_mode(rendering::TextureWrapping::ClampToBorder)
            .generate_texture(Vec::new());
    }
    // Update the last frame variable and dispatch the compute shader
    pub fn generate_voxels_start(&self, event_data: &mut SystemEventData, size: &u64, position: &veclib::Vector3<i64>) {
        // Get the compute shader
        let compute = event_data.shader_cacher.1.get_object_mut(self.compute_shader_name.as_str()).unwrap();

        // Set the compute shader variables and voxel texture
        compute.use_shader();
        compute.set_i3d("voxel_image", &self.voxel_texture, rendering::TextureShaderAccessType::ReadWrite);
        compute.set_i32("chunk_size", &(CHUNK_SIZE as i32));
        compute.set_vec3f32("node_pos", &veclib::Vector3::<f32>::from(*position));
        compute.set_i32("node_size", &(*size as i32));

        // Run the compute shader
        let compute_shader = match &mut compute.additional_shader {
            rendering::AdditionalShader::Compute(c) => c,
            _ => todo!(),
        };
        // Dispatch the compute shader, don't read back the data imme
        compute_shader.run_compute((CHUNK_SIZE as u32, CHUNK_SIZE as u32, CHUNK_SIZE as u32));
    }
    // Read back the data from the compute shader
    pub fn generate_voxels_end(&self, event_data: &mut SystemEventData, data: &mut Box<[Voxel]>) -> Option<()> {
        // Get the compute shader
        let compute = event_data.shader_cacher.1.get_object_mut(self.compute_shader_name.as_str()).unwrap();

        // Run the compute shader
        let compute_shader = match &mut compute.additional_shader {
            rendering::AdditionalShader::Compute(c) => c,
            _ => todo!(),
        };
        // Read back the compute shader data
        compute_shader.get_compute_state();
        // Read back the texture into the data buffer
        let pixels = self.voxel_texture.internal_texture.fill_array_elems::<f32>();

        // Keep track of the min and max values
        let mut min = f32::MAX;
        let mut max = f32::MIN;

        // Turn the pixels into the data
        for (i, pixel) in pixels.iter().enumerate() {
            let density = *pixel;
            data[i] = Voxel { density: density };
            min = min.min(density);
            max = max.max(density);
        }
        // Only generate the mesh if we have a surface
        if min.signum() != max.signum() {
            return Some(());
        } else {
            return None;
        }
    }
}

// Just a simple voxel
#[derive(Default, Clone, Copy)]
pub struct Voxel {
    pub density: f32,
}
