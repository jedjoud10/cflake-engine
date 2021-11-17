use crate::{utils, ISOLINE, MAIN_CHUNK_SIZE};
use rendering::advanced::*;
use rendering::basics::*;
use rendering::utils::*;
// Just a simple voxel
#[derive(Default, Clone, Copy)]
pub struct Voxel {
    pub density: f32,
    pub normal: veclib::Vector3<f32>,
    pub shader_id: u8,
    // Voxel material (Not actual renderer material)
    pub material_id: u8,
}
// Handles the generation of voxel data
#[derive(Default)]
pub struct VoxelGenerator {
    // The compute shader's name used for voxel generation
    pub compute: Shader,
    // The 3D texture used for voxel generation, only stores the density in a 16 bit value
    pub voxel_texture: Texture,
    // The 3D texture used to store MaterialID, BiomeID, Hardness and Smoothness
    pub material_texture: Texture,
}

impl VoxelGenerator {
    // New
    pub fn new(compute: Shader) -> Self {
        Self { compute, ..Self::default() }
    }
    // Generate the voxel texture
    pub fn setup_voxel_generator(&mut self) {
        // Create the voxel texture
        self.voxel_texture = Texture::default()
            .set_dimensions(TextureType::Texture3D(
                (MAIN_CHUNK_SIZE + 2) as u16,
                (MAIN_CHUNK_SIZE + 2) as u16,
                (MAIN_CHUNK_SIZE + 2) as u16,
            ))
            .set_format(TextureFormat::R16F)
            .set_data_type(DataType::Float32)
            .set_filter(TextureFilter::Nearest)
            .set_wrapping_mode(TextureWrapping::ClampToBorder)
            .generate_texture(Vec::new())
            .unwrap();
        self.material_texture = Texture::default()
            .set_dimensions(TextureType::Texture3D(
                (MAIN_CHUNK_SIZE + 2) as u16,
                (MAIN_CHUNK_SIZE + 2) as u16,
                (MAIN_CHUNK_SIZE + 2) as u16,
            ))
            .set_format(TextureFormat::RG8R)
            .set_filter(TextureFilter::Nearest)
            .set_wrapping_mode(TextureWrapping::ClampToBorder)
            .generate_texture(Vec::new())
            .unwrap();
    }
    // Update the last frame variable and dispatch the compute shader
    pub fn generate_voxels_start(&mut self, size: u64, depth: u8, position: veclib::Vector3<i64>) {
        //println!("Start voxel generation");
        // First pass
        let shader = &mut self.compute;
        shader.use_shader();
        shader.set_i3d("voxel_image", &self.voxel_texture, TextureShaderAccessType::WriteOnly);
        shader.set_i3d("material_image", &self.material_texture, TextureShaderAccessType::WriteOnly);
        shader.set_i32("chunk_size", &((MAIN_CHUNK_SIZE + 2) as i32));
        shader.set_vec3f32("node_pos", &veclib::Vector3::<f32>::from(position));
        shader.set_i32("node_size", &(size as i32));
        shader.set_i32("depth", &(depth as i32));
        // Run the compute shader
        let compute = match &mut shader.additional_shader {
            AdditionalShader::Compute(c) => c,
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
    pub fn generate_voxels_end(&mut self, _size: u64, _depth: u8, _position: veclib::Vector3<i64>) -> Option<Box<[Voxel]>> {
        //println!("End voxel generation");
        let shader = &mut self.compute;
        shader.use_shader();
        let compute = match &mut shader.additional_shader {
            AdditionalShader::Compute(c) => c,
            _ => panic!(),
        };

        // Read back the compute shader data
        compute.get_compute_state().unwrap();
        // Read back the texture into the data buffer
        let voxel_pixels = self.voxel_texture.fill_array_elems::<f32>();
        let material_pixels = self.material_texture.fill_array_veclib::<veclib::Vector2<u8>, u8>();
        // Keep track of the min and max values
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        // Turn the pixels into the data
        let mut local_data: Box<[(f32, u8, u8)]> = vec![(0.0, 0, 0); (MAIN_CHUNK_SIZE + 2) * (MAIN_CHUNK_SIZE + 2) * (MAIN_CHUNK_SIZE + 2)].into_boxed_slice();
        let mut data: Box<[Voxel]> = vec![Voxel::default(); (MAIN_CHUNK_SIZE + 1) * (MAIN_CHUNK_SIZE + 1) * (MAIN_CHUNK_SIZE + 1)].into_boxed_slice();
        for (i, pixel) in voxel_pixels.iter().enumerate() {
            let density = *pixel;
            let material = material_pixels[i];
            // Keep the min and max
            min = min.min(density);
            max = max.max(density);
            // Create the simplified voxel
            let simplified_voxel_tuple = (density, material.x, material.y);
            local_data[i] = simplified_voxel_tuple;
        }
        // Flatten using the custom size of MAIN_CHUNK_SIZE+2
        fn custom_flatten(x: usize, y: usize, z: usize) -> usize {
            x + (y * (MAIN_CHUNK_SIZE + 2) * (MAIN_CHUNK_SIZE + 2)) + (z * (MAIN_CHUNK_SIZE + 2))
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
                    let normal = veclib::Vector3::new(v1.0 as f32 - v0.0 as f32, v2.0 as f32 - v0.0 as f32, v3.0 as f32 - v0.0 as f32).normalized();
                    let sv = local_data[i];
                    let voxel = Voxel {
                        density: sv.0,
                        normal,
                        shader_id: sv.1,
                        material_id: sv.2,
                    };
                    data[utils::flatten((x, y, z))] = voxel;
                }
            }
        }
        // Only generate the mesh if we have a surface
        if (min < ISOLINE) != (max < ISOLINE) {
            Some(data)
        } else {
            None
        }
    }
}
