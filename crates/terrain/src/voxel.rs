use crate::{utils, ISOLINE, MAIN_CHUNK_SIZE};
use assets::AssetManager;
use rendering::{Shader, Texture, TextureFilter, TextureType};
use std::time::Instant;
use veclib::Swizzable;

// Just a simple voxel
#[derive(Default, Clone, Copy)]
pub struct Voxel {
    pub density: f32,
    pub color: veclib::Vector3<u8>,
    pub normal: veclib::Vector3<f32>,
    pub shader_id: u8,
    pub texture_id: u8,
    pub biome_id: u8,
    pub hardness: u8,
    // Very hard ( ͡° ͜ʖ ͡°)
}
// A skirt voxel
#[derive(Default, Clone, Copy)]
pub struct SkirtVoxel {
    pub density: f32,
    pub color: veclib::Vector3<u8>,
    pub shader_id: u8,
    pub texture_id: u8,
}
// Handles the generation of voxel data
#[derive(Default)]
pub struct VoxelGenerator {
    // The compute shader's name used for voxel generation
    pub compute: Shader,
    // The seconday compute shader's name
    pub color_compute: Shader,
    // The 3D texture used for voxel generation, only stores the density in a 16 bit value
    pub voxel_texture: Texture,
    // The 3D texture used to store MaterialID, BiomeID, Hardness and Smoothness
    pub material_texture: Texture,
    // The secondary 3D texture, stores the color (RGB)
    pub color_texture: Texture,
}

impl VoxelGenerator {
    // Generate the voxel texture
    pub fn setup_voxel_generator(&mut self) {
        // Create the voxel texture
        self.voxel_texture = Texture::new()
            .set_dimensions(TextureType::Texture3D(
                (MAIN_CHUNK_SIZE + 2) as u16,
                (MAIN_CHUNK_SIZE + 2) as u16,
                (MAIN_CHUNK_SIZE + 2) as u16,
            ))
            .set_idf(gl::R32F, gl::RED, gl::FLOAT)
            .set_filter(TextureFilter::Nearest)
            .set_wrapping_mode(rendering::TextureWrapping::ClampToBorder)
            .generate_texture(Vec::new())
            .unwrap();
        self.material_texture = Texture::new()
            .set_dimensions(TextureType::Texture3D(
                (MAIN_CHUNK_SIZE + 2) as u16,
                (MAIN_CHUNK_SIZE + 2) as u16,
                (MAIN_CHUNK_SIZE + 2) as u16,
            ))
            .set_idf(gl::RGBA8, gl::RGBA, gl::UNSIGNED_BYTE)
            .set_wrapping_mode(rendering::TextureWrapping::ClampToBorder)
            .generate_texture(Vec::new())
            .unwrap();
        self.color_texture = Texture::new()
            .set_dimensions(TextureType::Texture3D(
                (MAIN_CHUNK_SIZE + 2) as u16,
                (MAIN_CHUNK_SIZE + 2) as u16,
                (MAIN_CHUNK_SIZE + 2) as u16,
            ))
            .set_idf(gl::RGBA8, gl::RGBA, gl::UNSIGNED_BYTE)
            .set_wrapping_mode(rendering::TextureWrapping::ClampToBorder)
            .generate_texture(Vec::new())
            .unwrap();
    }
    // Update the last frame variable and dispatch the compute shader
    pub fn generate_voxels_start(&mut self, size: u64, depth: u8, position: veclib::Vector3<i64>) {
        // First pass
        let shader = &mut self.compute;
        shader.use_shader();
        shader.set_i3d("voxel_image", &self.voxel_texture, rendering::TextureShaderAccessType::WriteOnly);
        shader.set_i3d("material_image", &self.material_texture, rendering::TextureShaderAccessType::WriteOnly);
        shader.set_i32("chunk_size", &((MAIN_CHUNK_SIZE + 2) as i32));
        shader.set_vec3f32("node_pos", &veclib::Vector3::<f32>::from(position));
        shader.set_i32("node_size", &(size as i32));
        shader.set_i32("depth", &(depth as i32));
        // Run the compute shader
        let compute = match &mut shader.additional_shader {
            rendering::AdditionalShader::Compute(c) => c,
            _ => todo!(),
        };
        // Dispatch the compute shader, don't read back the data immediatly
        compute
            .run_compute((
                (MAIN_CHUNK_SIZE + 2) as u32 / 8 + 1,
                (MAIN_CHUNK_SIZE + 2) as u32 / 8 + 1,
                (MAIN_CHUNK_SIZE + 2) as u32 / 8 + 1,
            ))
            .unwrap();
    }
    // Read back the data from the compute shader
    pub fn generate_voxels_end(&mut self, size: u64, depth: u8, position: veclib::Vector3<i64>) -> (bool, Box<[Voxel]>) {
        let shader = &mut self.compute;
        shader.use_shader();
        let compute = match &mut shader.additional_shader {
            rendering::AdditionalShader::Compute(c) => c,
            _ => panic!(),
        };

        // Read back the compute shader data
        compute.get_compute_state().unwrap();
        // Second pass
        let color_shader = &mut self.color_compute;
        color_shader.use_shader();
        color_shader.set_i3d("color_image", &self.color_texture, rendering::TextureShaderAccessType::WriteOnly);
        color_shader.set_t3d("voxel_sampler", &self.voxel_texture, 1);
        color_shader.set_t3d("material_sampler", &self.material_texture, 2);
        color_shader.set_i32("chunk_size", &((MAIN_CHUNK_SIZE + 2) as i32));
        color_shader.set_vec3f32("node_pos", &veclib::Vector3::<f32>::from(position));
        color_shader.set_i32("node_size", &(size as i32));
        color_shader.set_i32("depth", &(depth as i32));
        let color_compute = match &mut color_shader.additional_shader {
            rendering::AdditionalShader::Compute(c) => c,
            _ => todo!(),
        };
        // Run the color compute shader
        let i = Instant::now();
        color_compute
            .run_compute((
                (MAIN_CHUNK_SIZE + 2) as u32 / 8 + 1,
                (MAIN_CHUNK_SIZE + 2) as u32 / 8 + 1,
                (MAIN_CHUNK_SIZE + 2) as u32 / 8 + 1,
            ))
            .unwrap();
        color_compute.get_compute_state().unwrap();
        // Read back the texture into the data buffer
        let voxel_pixels = self.voxel_texture.fill_array_elems::<f32>();
        let material_pixels = self.material_texture.fill_array_veclib::<veclib::Vector4<u8>, u8>();
        let color_pixels = self.color_texture.fill_array_veclib::<veclib::Vector4<u8>, u8>();
        //println!("{}", i.elapsed().as_millis());
        // Keep track of the min and max values
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        // Turn the pixels into the data
        let mut local_data: Box<[Voxel]> = Box::new([Voxel::default(); (MAIN_CHUNK_SIZE + 2) * (MAIN_CHUNK_SIZE + 2) * (MAIN_CHUNK_SIZE + 2)]);
        let mut data: Box<[Voxel]> = Box::new([Voxel::default(); (MAIN_CHUNK_SIZE + 1) * (MAIN_CHUNK_SIZE + 1) * (MAIN_CHUNK_SIZE + 1)]);
        for (i, pixel) in voxel_pixels.iter().enumerate() {
            let density = *pixel;
            let material = material_pixels[i];
            local_data[i] = Voxel {
                density: density,
                color: color_pixels[i].get3([0, 1, 2]),
                shader_id: material.x,
                texture_id: material.y,
                normal: veclib::Vector3::ZERO, // We are going to calculate the voxel normal in the next step
                biome_id: material.z,
                hardness: material.w,
            };
            // Keep the min and max
            min = min.min(density);
            max = max.max(density);
        }
        // Flatten using the custom size of MAIN_CHUNK_SIZE+2
        fn custom_flatten(x: usize, y: usize, z: usize) -> usize {
            return x + (y * (MAIN_CHUNK_SIZE + 2) * (MAIN_CHUNK_SIZE + 2)) + (z * (MAIN_CHUNK_SIZE + 2));
        }
        // Calculate the voxel normal
        for x in 0..(MAIN_CHUNK_SIZE + 1) {
            for y in 0..(MAIN_CHUNK_SIZE + 1) {
                for z in 0..(MAIN_CHUNK_SIZE + 1) {
                    let i = custom_flatten(x, y, z);
                    let v0 = local_data[i];
                    // Calculate the normal using the difference between neigboring voxels
                    let v1 = local_data[custom_flatten(x + 1, y, z)];
                    let v2 = local_data[custom_flatten(x, y + 1, z)];
                    let v3 = local_data[custom_flatten(x, y, z + 1)];
                    // Normal
                    let normal = veclib::Vector3::new(
                        v1.density as f32 - v0.density as f32,
                        v2.density as f32 - v0.density as f32,
                        v3.density as f32 - v0.density as f32,
                    )
                    .normalized();
                    let mut voxel = local_data[i];
                    voxel.normal = normal;
                    data[utils::flatten((x, y, z))] = voxel;
                }
            }
        }
        // Only generate the mesh if we have a surface
        ((min < ISOLINE) != (max < ISOLINE), data)
    }
}
