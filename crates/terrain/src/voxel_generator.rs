use rendering::{GPUObjectID, pipec, TextureWrapping, TextureFilter, TextureFormat, TextureType};

use crate::{DEFAULT_TERRAIN_COMPUTE_SHADER, MAIN_CHUNK_SIZE, ChunkCoords};

// Handles the generation of voxel data
#[derive(Default)]
pub struct VoxelGenerator {    
    pub compute: GPUObjectID, // The compute shader that is used for voxel generation    
    pub voxel_texture: GPUObjectID, // The 3D texture used for voxel generation, only stores the density in a 16 bit value    
    pub material_texture: GPUObjectID, // The 3D texture used to store MaterialID, ShaderID
    pub pending_generation: Vec<ChunkCoords> // The chunks that are pending their voxel generation
}

impl VoxelGenerator {
    // New
    pub fn new(interpreter_string: String) -> Self {
        // Create the voxel generation shader
        let compute = pipec::compute_shader(
            rendering::Shader::default()
                .load_externalcode("voxel_interpreter", interpreter_string)
                .load_shader(vec![DEFAULT_TERRAIN_COMPUTE_SHADER])
                .unwrap(),
        );
        // Create the voxel texture
        let voxel_texture = pipec::texture(
            rendering::Texture::default()
                .set_dimensions(TextureType::Texture3D(
                    (MAIN_CHUNK_SIZE + 2) as u16,
                    (MAIN_CHUNK_SIZE + 2) as u16,
                    (MAIN_CHUNK_SIZE + 2) as u16,
                ))
                .set_format(TextureFormat::R16F)
                .set_data_type(rendering::DataType::Float32)
                .set_filter(TextureFilter::Nearest)
                .set_wrapping_mode(TextureWrapping::ClampToBorder),
        );
        let material_texture = pipec::texture(
            rendering::Texture::default()
                .set_dimensions(TextureType::Texture3D(
                    (MAIN_CHUNK_SIZE + 2) as u16,
                    (MAIN_CHUNK_SIZE + 2) as u16,
                    (MAIN_CHUNK_SIZE + 2) as u16,
                ))
                .set_format(TextureFormat::RG8R)
                .set_filter(TextureFilter::Nearest)
                .set_wrapping_mode(TextureWrapping::ClampToBorder),
        );
        // Create self
        Self {
            compute,
            voxel_texture,
            material_texture,
            pending_generation: Vec::new()
        }
    }
    // We must generate the voxel data for this specific chunk, but not immediately. We must buffer it
    pub fn generate_voxel_data(chunk_coords: ChunkCoords) {

    } 
    /*
    // Update the last frame variable and dispatch the compute shader
    pub fn generate_voxels_start(&mut self, size: u64, depth: u8, position: veclib::Vector3<i64>) {
        // First pass
        let mut group = rendering::ShaderUniformsGroup::new();
        group.update_shader_id(&self.compute);
        group.set_i3d("voxel_image", &self.voxel_texture, TextureShaderAccessType::WriteOnly);
        group.set_i3d("material_image", &self.material_texture, TextureShaderAccessType::WriteOnly);
        group.set_i32("chunk_size", (MAIN_CHUNK_SIZE + 2) as i32);
        group.set_vec3f32("node_pos", veclib::Vector3::<f32>::from(position));
        group.set_i32("node_size", size as i32);
        group.set_i32("depth", depth as i32);
        // Dispatch the compute shader, don't read back the data immediatly
        let x = ((MAIN_CHUNK_SIZE + 2) as u16 / 8 + 1,
        (MAIN_CHUNK_SIZE + 2) as u16 / 8 + 1,
        (MAIN_CHUNK_SIZE + 2) as u16 / 8 + 1,);
        let result = pipec::task(pipec::RenderTask::ComputeRun(self.compute, x, group));
        result.wait_execution();
    }
    // Read back the data from the compute shader
    pub fn generate_voxels_end(&mut self, _size: u64, _depth: u8, _position: veclib::Vector3<i64>) -> Option<Box<[Voxel]>> {
        // Read back the compute shader data
        pipec::task(pipec::RenderTask::ComputeLock(self.compute)).wait_execution();
        // Read back the texture into the data buffer
        // Wait for main voxel gen
        let mut voxel_pixels: Vec<u8> = Vec::new();
        let result1 = pipec::task(pipeline::RenderTask::TextureFillArray(self.voxel_texture, std::mem::size_of::<f32>(), Arc::new(AtomicPtr::new(&mut voxel_pixels))));
        result1.wait_execution();
        // Wait for secondary voxel gen
        let mut material_pixels: Vec<u8> = Vec::new();
        let result2 = pipec::task(pipeline::RenderTask::TextureFillArray(self.material_texture, std::mem::size_of::<u8>() * 2, Arc::new(AtomicPtr::new(&mut material_pixels))));
        result2.wait_execution();
        let voxel_pixels = pipec::convert_native::<f32>(voxel_pixels);
        let material_pixels = pipec::convert_native_veclib::<veclib::Vector2<u8>, u8>(material_pixels);
        // Keep track of the min and max values
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        // Turn the pixels into the data
        let mut local_data: Box<[(f32, u8, u8)]> = vec![(0.0, 0, 0); (MAIN_CHUNK_SIZE + 2) * (MAIN_CHUNK_SIZE + 2) * (MAIN_CHUNK_SIZE + 2)].into_boxed_slice();
        let mut data: Box<[Voxel]> = vec![Voxel::default(); (MAIN_CHUNK_SIZE + 1) * (MAIN_CHUNK_SIZE + 1) * (MAIN_CHUNK_SIZE + 1)].into_boxed_slice();
        for (i, density) in voxel_pixels.into_iter().enumerate() {
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
    */
}
