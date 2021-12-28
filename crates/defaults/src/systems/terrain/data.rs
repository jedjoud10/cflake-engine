use rendering::{pipec, GPUObjectID, TextureFilter, TextureFormat, TextureType, TextureWrapping};
use std::collections::{HashMap, HashSet};
use terrain::{ChunkCoords, ChunkState, VoxelData, DEFAULT_TERRAIN_COMPUTE_SHADER, MAIN_CHUNK_SIZE};

// Handles the creation/destruction of the chunk entities
#[derive(Default)]
pub struct ChunkSystem {
    pub octree: math::octrees::AdvancedOctree,          // An advanced octree, so we can actually create the chunks
    pub csgtree: math::csg::CSGTree,                    // The CSG tree that will be used for massive optimizations
    pub chunks: HashMap<ChunkCoords, usize>,            // The chunks that were added into the world
    pub chunk_states: HashMap<ChunkCoords, ChunkState>, // The chunks and their current state
}

// Handles the voxel generation for each chunk
#[derive(Default)]
pub struct VoxelGenerationSystem {
    pub compute: GPUObjectID,                             // The compute shader that is used for voxel generation
    pub voxel_texture: GPUObjectID,                       // The 3D texture used for voxel generation, only stores the density in a 16 bit value
    pub material_texture: GPUObjectID,                    // The 3D texture used to store MaterialID, ShaderID
    pub result: Option<(ChunkCoords, Option<VoxelData>)>, // The voxel data that we generated. Also contains the ChunkCoords of the matching Chunk
    pub generating: bool,                                 // Are we currently generating / waiting for the voxel data?
    pub pending_chunks: Vec<ChunkCoords>,                 // The chunks that are pending their voxel data generation
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
}

// Handles the mesh generation for each chunk
#[derive(Default)]
pub struct MesherSystem {
    pub material: rendering::GPUObjectID,     // The Chunks' terrain material
    pub pending_chunks: HashSet<ChunkCoords>, // The chunks that are pending their mesh generation
}
