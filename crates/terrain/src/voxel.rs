use crate::ISOLINE;

use super::CHUNK_SIZE;
use others::CacheManager;
use rendering::{Shader, Texture, TextureDimensions};
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

// Just a simple voxel
#[derive(Default, Clone, Copy)]
pub struct Voxel {
    pub color: veclib::Vector3<u8>,
    pub density: u16,
    pub biome_id: u8,
    pub material_id: u8,
    // Very hard ( ͡° ͜ʖ ͡°)
    pub hardness: u8,
    pub texture_id: u8,
}
// Handles the generation of voxel data
#[derive(Default)]
pub struct VoxelGenerator {
    // The compute shader's name used for voxel generation
    pub compute_id: usize,
    // The seconday compute shader's name
    pub color_compute_id: usize,
    // The 3D texture used for voxel generation, only stores the density in a 16 bit value
    pub voxel_texture: usize,
    // The 3D texture used to store MaterialID, BiomeID, Hardness and Smoothness
    pub material_texture: usize,
    // The secondary 3D texture, stores the color (RGB)
    pub color_texture: usize,
}

impl VoxelGenerator {
    // Generate the voxel texture
    pub fn setup_voxel_generator(&mut self, texture_cacher: &mut CacheManager<Texture>) {
        // Create the voxel texture
        self.voxel_texture = Texture::new()
            .set_dimensions(TextureDimensions::D3D(CHUNK_SIZE as u16, CHUNK_SIZE as u16, CHUNK_SIZE as u16))
            .set_idf(gl::R16, gl::RED, gl::UNSIGNED_BYTE)
            .set_wrapping_mode(rendering::TextureWrapping::ClampToBorder)
            .generate_texture(Vec::new())
            .cache_texture(texture_cacher)
            .unwrap()
            .1;
        self.material_texture = Texture::new()
            .set_dimensions(TextureDimensions::D3D(CHUNK_SIZE as u16, CHUNK_SIZE as u16, CHUNK_SIZE as u16))
            .set_idf(gl::RGBA8, gl::RGBA, gl::UNSIGNED_BYTE)
            .set_wrapping_mode(rendering::TextureWrapping::ClampToBorder)
            .generate_texture(Vec::new())
            .cache_texture(texture_cacher)
            .unwrap()
            .1;
        self.color_texture = Texture::new()
            .set_dimensions(TextureDimensions::D3D(CHUNK_SIZE as u16, CHUNK_SIZE as u16, CHUNK_SIZE as u16))
            .set_idf(gl::RGB8, gl::RGB, gl::UNSIGNED_BYTE)
            .set_wrapping_mode(rendering::TextureWrapping::ClampToBorder)
            .generate_texture(Vec::new())
            .cache_texture(texture_cacher)
            .unwrap()
            .1;
    }
    // Update the last frame variable and dispatch the compute shader
    pub fn generate_voxels_start(&self, shader_cacher: &mut CacheManager<Shader>, size: &u64, position: &veclib::Vector3<i64>) {
        // First pass
        // Set the compute shader variables and voxel texture
        let shader = shader_cacher.id_get_object_mut(self.compute_id).unwrap();
        let vals = vec![
            (
                "voxel_image",
                rendering::Uniform::Image3D(self.voxel_texture, rendering::TextureShaderAccessType::WriteOnly),
            ),
            (
                "material_image",
                rendering::Uniform::Image3D(self.material_texture, rendering::TextureShaderAccessType::WriteOnly),
            ),
            ("chunk_size", rendering::Uniform::I32(CHUNK_SIZE as i32)),
            ("node_pos", rendering::Uniform::Vec3F32(veclib::Vector3::<f32>::from(*position))),
            ("node_size", rendering::Uniform::I32(*size as i32)),
        ];
        // Run the compute shader
        let compute = match &mut shader.additional_shader {
            rendering::AdditionalShader::Compute(c) => c,
            _ => todo!(),
        };
        // Dispatch the compute shader, don't read back the data immediatly
        compute.run_compute((CHUNK_SIZE as u32 / 4, CHUNK_SIZE as u32 / 4, CHUNK_SIZE as u32 / 4)).unwrap();
    }
    // Read back the data from the compute shader
    pub fn generate_voxels_end(
        &self,
        shader_cacher: &mut CacheManager<Shader>,
        texture_cacher: &CacheManager<Texture>,
        size: &u64,
        position: &veclib::Vector3<i64>,
        data: &mut Box<[Voxel]>,
    ) -> bool {
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
        let vals = vec![
            (
                "color_image",
                rendering::Uniform::Image3D(self.color_texture, rendering::TextureShaderAccessType::WriteOnly),
            ),
            ("voxel_sampler", rendering::Uniform::Texture3D(self.voxel_texture, gl::TEXTURE1)),
            ("material_sampler", rendering::Uniform::Texture3D(self.voxel_texture, gl::TEXTURE2)),
            (
                "material_image",
                rendering::Uniform::Image3D(self.material_texture, rendering::TextureShaderAccessType::WriteOnly),
            ),
            ("chunk_size", rendering::Uniform::I32(CHUNK_SIZE as i32)),
            ("node_pos", rendering::Uniform::Vec3F32(veclib::Vector3::<f32>::from(*position))),
            ("node_size", rendering::Uniform::I32(*size as i32)),
        ];
        color_shader.set_vals(vals, texture_cacher).unwrap();
        let color_compute = match &mut color_shader.additional_shader {
            rendering::AdditionalShader::Compute(c) => c,
            _ => todo!(),
        };
        // Run the color compute shader
        color_compute.run_compute((CHUNK_SIZE as u32 / 4, CHUNK_SIZE as u32 / 4, CHUNK_SIZE as u32 / 4)).unwrap();
        color_compute.get_compute_state().unwrap();

        // Read back the texture into the data buffer
        // Voxel
        let voxel_texture = texture_cacher.id_get_object(self.voxel_texture).unwrap();
        let voxel_pixels = voxel_texture.fill_array_elems::<u16>();
        // Material
        let material_texture = texture_cacher.id_get_object(self.material_texture).unwrap();
        let material_pixels = material_texture.fill_array_veclib::<veclib::Vector4<u8>, u8>();
        // Color
        let color_texture = texture_cacher.id_get_object(self.voxel_texture).unwrap();
        let color_pixels = color_texture.fill_array_veclib::<veclib::Vector4<u8>, u8>();
        // Keep track of the min and max values
        let mut min = u16::MAX;
        let mut max = u16::MIN;
        // Turn the pixels into the data
        for (i, pixel) in voxel_pixels.iter().enumerate() {
            let density = *pixel;
            let material_data = material_pixels[i];
            let color = color_pixels[i];
            data[i] = Voxel {
                density: density,
                color: color.get3([0, 1, 2]),
                biome_id: material_data.x,
                material_id: material_data.y,
                hardness: material_data.z,
                texture_id: material_data.w,
            };
            // Keep the min and max
            min = min.min(density);
            max = max.max(density);
        }
        // Only generate the mesh if we have a surface
        (min < ISOLINE) != (max < ISOLINE)
    }
}
