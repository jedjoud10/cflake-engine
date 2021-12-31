use ecs::EntityID;
use rendering::{pipec, GPUObjectID, TextureFilter, TextureFormat, TextureType, TextureWrapping};
use std::collections::{HashMap, HashSet};
use terrain::{ChunkCoords, VoxelData, DEFAULT_TERRAIN_COMPUTE_SHADER, MAIN_CHUNK_SIZE};

// Handles the creation/destruction of the chunk entities
#[derive(Default)]
pub struct ChunkSystem {
    pub octree: math::octrees::AdvancedOctree,  // An advanced octree, so we can actually create the chunks
    pub csgtree: math::csg::CSGTree,            // The CSG tree that will be used for massive optimizations
    pub chunks: HashMap<ChunkCoords, EntityID>, // The chunks that were added into the world
    pub chunks_to_delete: HashSet<EntityID>, // The chunks that we must delete
    pub deleted_chunks_descending: HashSet<EntityID>, // The chunks that have been delete will remove their EntityID from this set
    pub chunks_awaiting_validation: HashSet<ChunkCoords>, // The number of chunks that are awating to be created and validated
    pub removal_time: f32, // The moment in seconds since the start of the game when we want to delete the chunks
}

pub const PARALLEL_COMPUTES: usize = 2; // The number of computes shaders that are ran in parallel

// Handles the voxel generation for each chunk
#[derive(Default)]
pub struct VoxelGenerationSystem {
    pub computes: Vec<(GPUObjectID, bool)>,                       // The computes shaders that are used for voxel generation
    pub voxel_texture: Vec<GPUObjectID>,                          // The 3D texture used for voxel generation, only stores the density in a 16 bit value
    pub material_texture: Vec<GPUObjectID>,                       // The 3D texture used to store MaterialID, ShaderID
    pub pending_chunks: Vec<ChunkCoords>,                         // The chunks that are pending their voxel data generation
    pub results: HashMap<ChunkCoords, Option<Option<VoxelData>>>, // A specific result for a specific chunk
}
impl VoxelGenerationSystem {
    pub fn new(interpreter_string: String) -> Self {
        // Create the voxel generation shader
        fn create_compute(interpreter_string: String, i: usize) -> GPUObjectID {
            pipec::compute_shader(
                rendering::Shader::default()
                    .load_externalcode("voxel_interpreter", interpreter_string)
                    .load_shader(vec![DEFAULT_TERRAIN_COMPUTE_SHADER])
                    .unwrap()
                    .prefix_name(&i.to_string()),
            )
        }
        // Create the voxel texture
        fn create_voxel_texture(i: usize) -> GPUObjectID {
            pipec::texture(
                rendering::Texture::default()
                    .set_dimensions(TextureType::Texture3D(
                        (MAIN_CHUNK_SIZE + 2) as u16,
                        (MAIN_CHUNK_SIZE + 2) as u16,
                        (MAIN_CHUNK_SIZE + 2) as u16,
                    ))
                    .set_format(TextureFormat::R16F)
                    .set_data_type(rendering::DataType::Float32)
                    .set_filter(TextureFilter::Nearest)
                    .set_wrapping_mode(TextureWrapping::ClampToBorder)
                    .prefix_name(&i.to_string()),
            )
        }
        // Create the voxel texture
        fn create_material_texture(i: usize) -> GPUObjectID {
            pipec::texture(
                rendering::Texture::default()
                    .set_dimensions(TextureType::Texture3D(
                        (MAIN_CHUNK_SIZE + 2) as u16,
                        (MAIN_CHUNK_SIZE + 2) as u16,
                        (MAIN_CHUNK_SIZE + 2) as u16,
                    ))
                    .set_format(TextureFormat::RG8R)
                    .set_filter(TextureFilter::Nearest)
                    .set_wrapping_mode(TextureWrapping::ClampToBorder)
                    .prefix_name(&i.to_string()),
            )
        }
        // Create self
        let computes = (0_usize..PARALLEL_COMPUTES)
            .into_iter()
            .map(|i: usize| (create_compute(interpreter_string.clone(), i), false))
            .collect::<Vec<(GPUObjectID, bool)>>();
        let vtextures = (0_usize..PARALLEL_COMPUTES)
            .into_iter()
            .map(|i: usize| create_voxel_texture(i))
            .collect::<Vec<GPUObjectID>>();
        let mtextures = (0_usize..PARALLEL_COMPUTES)
            .into_iter()
            .map(|i: usize| create_material_texture(i))
            .collect::<Vec<GPUObjectID>>();
        Self {
            computes,
            voxel_texture: vtextures,
            material_texture: mtextures,
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
