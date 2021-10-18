use crate::ISOLINE;

use super::CHUNK_SIZE;
use others::CacheManager;
use rendering::{Shader, Texture2D, Texture3D};
use veclib::Swizzable;
use world_data::WorldData;

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
    pub compute_id: usize,
    // The seconday compute shader's name
    pub color_compute_id: usize,
    // The 3D texture used for voxel generation as well
    pub voxel_texture: Texture3D,
    // The seconday 3D texture
    pub color_voxel_texture: Texture3D,
}

impl VoxelGenerator {
    // Generate the voxel texture
    pub fn setup_voxel_generator(&mut self) {
        // Create the voxel texture
        self.voxel_texture = Texture3D::new()
            .set_dimensions(CHUNK_SIZE as u16, CHUNK_SIZE as u16, CHUNK_SIZE as u16)
            .set_idf(gl::RGBA8, gl::RGBA, gl::UNSIGNED_BYTE)
            .set_wrapping_mode(rendering::TextureWrapping::ClampToBorder)
            .generate_texture(Vec::new());
        self.color_voxel_texture = Texture3D::new()
            .set_dimensions(CHUNK_SIZE as u16, CHUNK_SIZE as u16, CHUNK_SIZE as u16)
            .set_idf(gl::RGBA8, gl::RGBA, gl::UNSIGNED_BYTE)
            .set_wrapping_mode(rendering::TextureWrapping::ClampToBorder)
            .generate_texture(Vec::new());
    }
    // Update the last frame variable and dispatch the compute shader
    pub fn generate_voxels_start(&self, shader_cacher: &mut CacheManager<Shader>, size: &u64, position: &veclib::Vector3<i64>) {
        // First pass
        // Set the compute shader variables and voxel texture
        let shader = shader_cacher.id_get_object_mut(self.compute_id).unwrap();
        shader.use_shader();
        shader.set_i3d("voxel_image", &self.voxel_texture, rendering::TextureShaderAccessType::WriteOnly);
        shader.set_i32("chunk_size", &(CHUNK_SIZE as i32));
        shader.set_vec3f32("node_pos", &veclib::Vector3::<f32>::from(*position));
        shader.set_i32("node_size", &(*size as i32));
        // Run the compute shader
        let compute = match &mut shader.additional_shader {
            rendering::AdditionalShader::Compute(c) => c,
            _ => todo!(),
        };
        // Dispatch the compute shader, don't read back the data immediatly
        compute.run_compute((CHUNK_SIZE as u32 / 4, CHUNK_SIZE as u32 / 4, CHUNK_SIZE as u32 / 4)).unwrap();
    }
    // Read back the data from the compute shader
    pub fn generate_voxels_end(&self, shader_cacher: &mut CacheManager<Shader>, size: &u64, position: &veclib::Vector3<i64>, data: &mut Box<[Voxel]>) -> bool {
        let shader = shader_cacher.id_get_object_mut(self.compute_id).unwrap();
        shader.use_shader();
        let compute = match &mut shader.additional_shader {
            rendering::AdditionalShader::Compute(c) => c,
            _ => panic!(),
        };

        // Read back the compute shader data
        compute.get_compute_state().unwrap();

        // Dispatch the compute shader
        // Second pass
        let color_shader = shader_cacher.id_get_object_mut(self.color_compute_id).unwrap();
        color_shader.use_shader();
        color_shader.set_i3d("color_voxel_image", &self.color_voxel_texture, rendering::TextureShaderAccessType::WriteOnly);
        color_shader.set_t3d("voxel_sampler", &self.voxel_texture, gl::TEXTURE1);
        color_shader.set_i32("chunk_size", &(CHUNK_SIZE as i32));
        color_shader.set_vec3f32("node_pos", &veclib::Vector3::<f32>::from(*position));
        color_shader.set_i32("node_size", &(*size as i32));
        let color_compute = match &mut color_shader.additional_shader {
            rendering::AdditionalShader::Compute(c) => c,
            _ => todo!(),
        };
        // Run the color compute shader
        color_compute.run_compute((CHUNK_SIZE as u32 / 4, CHUNK_SIZE as u32 / 4, CHUNK_SIZE as u32 / 4)).unwrap();
        color_compute.get_compute_state().unwrap();

        // Read back the texture into the data buffer
        let pixels2 = self.color_voxel_texture.internal_texture.fill_array_veclib::<veclib::Vector4<u8>, u8>();
        let pixels = self.voxel_texture.internal_texture.fill_array_veclib::<veclib::Vector4<u8>, u8>();
        //let pixels = vec![0; data.len()];
        // Keep track of the min and max values
        let mut min = u16::MAX;
        let mut max = u16::MIN;

        // Turn the pixels into the data
        for (i, pixel) in pixels.iter().enumerate() {
            let voxel_data = *pixel;
            // Convert the two bytes into an i16
            let mut density: u16 = voxel_data.x as u16;
            density = density << 8;
            density |= voxel_data.y as u16;
            data[i] = Voxel {
                density: density,
                color: pixels2[i].get3([0, 1, 2]),
                biome_id: voxel_data.z,
                material_id: voxel_data.w,
            };
            min = min.min(density);
            max = max.max(density);
        }
        // Only generate the mesh if we have a surface
        (min < ISOLINE) != (max < ISOLINE)
    }
}

// Just a simple voxel
#[derive(Default, Clone, Copy)]
pub struct Voxel {
    pub color: veclib::Vector3<u8>,
    pub density: u16,
    pub biome_id: u8,
    pub material_id: u8,
}
