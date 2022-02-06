use main::{
    ecs::entity::EntityID,
    globals::Global,
    math::octrees::{DiffOctree, HeuristicSettings},
    rendering::{
        advanced::{
            atomic::{AtomicGroup, AtomicGroupRead, ClearCondition},
            compute::ComputeShader,
            shader_storage::ShaderStorage,
        },
        basics::{
            material::Material,
            readwrite::ReadBytes,
            shader::{self, ShaderSettings},
            transfer::Transferable,
            uniforms::ShaderUniformsGroup,
        },
        object::{ObjectID, ReservedTrackedID, TrackedTask},
        pipeline::{pipec, PipelineContext},
        utils::{AccessType, UpdateFrequency},
    },
    terrain::{ChunkCoords, PackedVoxel, PackedVoxelData, StoredVoxelData, CHUNK_SIZE},
};
use std::{
    collections::{HashMap, HashSet},
    mem::size_of,
};

mod chunks;
mod voxel_generation;
use chunks::ChunksHandler;
use voxel_generation::VoxelGenerator;

#[derive(Global)]
// The global terrain component that can be added at the start of the game
pub struct Terrain {
    // Handler for our chunks
    pub chunk_handler: ChunksHandler,
    // Handler for our voxel generation
    pub generator: VoxelGenerator,
}

impl Terrain {
    // Create a new terrain component
    pub fn new(voxel_src_path: &str, octree_depth: u8, pipeline_context: &PipelineContext) -> Self {
        // Create a new octree
        let octree = DiffOctree::new(octree_depth, (CHUNK_SIZE) as u64, HeuristicSettings::default());
        

        println!("Terrain component init done!");
        Self {
            chunk_handler: ChunksHandler::new(octree),
            generator: VoxelGenerator::new(voxel_src_path, pipeline_context),
        }
    }
    // Generate the terrain with a specific material
    pub fn set_material(mut self, material: ObjectID<Material>) -> Self {
        self.chunk_handler.material = material;
        self
    }
    // Generate the terrain with a specific octree heuristic settings
    pub fn set_heuristic(mut self, settings: HeuristicSettings) -> Self {
        self.chunk_handler.octree.update_heuristic(settings);
        self
    }
}
