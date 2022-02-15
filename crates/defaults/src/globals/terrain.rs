use main::{
    globals::Global,
    math::octrees::{DiffOctree, HeuristicSettings},
    rendering::{basics::material::Material, object::ObjectID, pipeline::PipelineContext},
    terrain::CHUNK_SIZE,
};

mod chunks;
mod voxel_generation;
mod settings;
pub use settings::*;
pub use chunks::ChunksHandler;
pub use voxel_generation::VoxelGenerator;

#[derive(Global)]
// The global terrain component that can be added at the start of the game
pub struct Terrain {
    // Handler for our chunks
    pub chunk_handler: ChunksHandler,
    // Handler for our voxel generation
    pub generator: VoxelGenerator,
    
    // Save the terrain settings for when we actually initialize the voxel generator
    settings: TerrainSettings,
}

impl Terrain {
    // Create a new terrain global
    pub fn new(settings: TerrainSettings, pipeline: &PipelineContext) -> Self {
        Self {
            chunk_handler: Default::default(),
            generator: VoxelGenerator::new(&settings.voxel_src_path, pipeline),
            settings,
        }
    }    
}
