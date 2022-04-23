use std::num::NonZeroU64;

use world::{
    resources::Resource,
    math::octrees::DiffOctree,
    rendering::pipeline::Pipeline,
    terrain::{
        editing::{Edit, EditingManager},
        scheduler::{MeshScheduler, MeshSchedulerSettings},
        ChunkCoords, CHUNK_SIZE,
    },
};

mod chunks_manager;
mod settings;
mod voxel_generation;
pub use chunks_manager::*;

pub use settings::*;
pub use voxel_generation::*;

#[derive(Resource)]
// The global terrain component that can be added at the start of the game
pub struct Terrain {
    // Handles chunk generation/deletion
    pub manager: ChunksManager,

    // Handles voxel generation on the GPU
    pub generator: VoxelGenerator,

    // Multithreaded mesh generation task system 
    pub scheduler: MeshScheduler,

    // Simply for editing
    pub editer: EditingManager,
}

impl Terrain {
    // Create a new terrain global
    pub fn new(settings: TerrainSettings, pipeline: &mut Pipeline) -> Self {
        Self {
            manager: ChunksManager {
                octree: DiffOctree::new(settings.depth, CHUNK_SIZE as u64, settings.heuristic_settings),
                material: settings.material,
                ..Default::default()
            },
            scheduler: MeshScheduler::new(MeshSchedulerSettings {
                // By default, the terrain will use 2 task-threads for mesh generation 
                thread_num: Some(2),
            }),
            generator: VoxelGenerator::new(&settings.voxel_src_path, pipeline),
            editer: EditingManager::default(),
        }
    }
    // Add a terrain edit
    pub fn edit(&mut self, edit: Edit) {
        self.editer.edit(edit);
    }
    // Force the re-generation of a specific chunk
    pub fn regenerate_chunk(&mut self, coords: ChunkCoords) -> Option<()> {
        // Check if the chunk is valid first
        if self.manager.chunks.contains_key(&coords) {
            // Regenerate
            if self.manager.chunks_generating.insert(coords) {
                // First time we queue this chunk for generation
                let id = self.manager.chunks.get(&coords)?;
                self.manager.priority_list.push((*id, 0.0));
            } else {
                // Already queued for generation
            }
            Some(())
        } else {
            // The chunk does not exist yet
            None
        }
    }
}
