use main::{globals::Global, math::octrees::DiffOctree, rendering::pipeline::Pipeline, terrain::CHUNK_SIZE};

mod chunks;
mod settings;
mod voxel_generation;
pub use chunks::ChunksHandler;
pub use settings::*;
pub use voxel_generation::VoxelGenerator;

#[derive(Global)]
// The global terrain component that can be added at the start of the game
pub struct Terrain {
    // Handler for our chunks
    pub chunk_handler: ChunksHandler,
    // Handler for our voxel generation
    pub generator: VoxelGenerator,
}

impl Terrain {
    // Create a new terrain global
    pub fn new(settings: TerrainSettings, pipeline: &Pipeline) -> Self {
        Self {
            chunk_handler: ChunksHandler {
                octree: DiffOctree::new(settings.depth, CHUNK_SIZE as u64, settings.heuristic_settings),
                material: settings.material,
                ..Default::default()
            },
            generator: VoxelGenerator::new(&settings.voxel_src_path, pipeline),
        }
    }
}
