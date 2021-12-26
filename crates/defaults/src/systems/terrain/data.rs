use core::global::callbacks::CallbackType;
use std::collections::HashMap;
use others::callbacks::OwnedCallback;
use rendering::{GPUObjectID, pipec, TextureFormat, TextureFilter, TextureWrapping, TextureType, TextureShaderAccessType};
use terrain::{ChunkCoords, VoxelData, DEFAULT_TERRAIN_COMPUTE_SHADER, MAIN_CHUNK_SIZE};

// Handles the creation/destruction of the chunk entities
#[derive(Default)]
pub struct ChunkSystem {
    pub octree: math::octrees::AdvancedOctree, // An advanced octree, so we can actually create the chunks
    pub material: rendering::GPUObjectID, // The Chunks' terrain material
    pub csgtree: math::csg::CSGTree, // The CSG tree that will be used for massive optimizations
    pub chunks: HashMap<ChunkCoords, usize>, // The chunks that were added into the world
}





// Handles the voxel generation for each chunk
#[derive(Default)]
pub struct VoxelGenerationSystem {
    compute: GPUObjectID, // The compute shader that is used for voxel generation    
    voxel_texture: GPUObjectID, // The 3D texture used for voxel generation, only stores the density in a 16 bit value    
    material_texture: GPUObjectID, // The 3D texture used to store MaterialID, ShaderID
    voxel_data: Option<VoxelData>, // The voxel data that we generated
    generating: bool, // Are we currently generating / waiting for the voxel data?
    pending_chunks: Vec<ChunkCoords>, // The chunks that are pending their voxel data generation
}
impl VoxelGenerationSystem {
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
            ..Self::default()
        }
    }
    // Are we free to do any additional calculations?
    pub fn is_free(&self) -> bool {
        !self.generating
    }
    // Add a pending chunk
    pub fn add_pending_chunk(&mut self, pending_chunk_coords: ChunkCoords) {
        self.pending_chunks.push(pending_chunk_coords);
    }
    // Get the first pending chunk
    pub fn remove_first_chunk(&mut self) -> ChunkCoords {
        self.pending_chunks.swap_remove(0)
    }
    // Start generating the voxel data for a specific chunk
    pub fn generate_voxel_data(&mut self, chunk_coords: ChunkCoords) {
        if !self.is_free() { panic!(); }
        // Set the state
        self.generating = true;

        // First pass
        let mut group = rendering::ShaderUniformsGroup::new();
        group.update_shader_id(&self.compute);
        group.set_i3d("voxel_image", &self.voxel_texture, TextureShaderAccessType::WriteOnly);
        group.set_i3d("material_image", &self.material_texture, TextureShaderAccessType::WriteOnly);
        group.set_i32("chunk_size", (MAIN_CHUNK_SIZE + 2) as i32);
        group.set_vec3f32("node_pos", veclib::Vector3::<f32>::from(chunk_coords.position));
        group.set_i32("node_size", chunk_coords.size as i32);
        group.set_i32("depth", chunk_coords.depth as i32);
        // Dispatch the compute shader, don't read back the data immediately
        let indices = ((MAIN_CHUNK_SIZE + 2) as u16 / 8 + 1,
        (MAIN_CHUNK_SIZE + 2) as u16 / 8 + 1,
        (MAIN_CHUNK_SIZE + 2) as u16 / 8 + 1,);
        let result = pipec::task(pipec::RenderTask::ComputeRun(self.compute, indices, group));
        // Callback data that we will pass
        let voxel_texture = self.voxel_texture.clone();
        result.with_callback(CallbackType::GPUObjectCallback(OwnedCallback::new(move |(gpuobject, gpuobject_id)| {
            // This callback is executed when the compute shader finishes it's execution. 
            // We can safely read back from the textures now
        })).create());
    }
}