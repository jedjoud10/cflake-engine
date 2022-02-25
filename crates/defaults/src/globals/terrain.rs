use std::sync::Arc;

use main::{
    globals::Global,
    math::octrees::DiffOctree,
    rendering::pipeline::Pipeline,
    terrain::{
        editing::{Edit, EditingManager},
        ChunkCoords, CHUNK_SIZE,
    },
};

mod chunks_manager;
mod settings;
mod voxel_generation;
pub use chunks_manager::*;
use parking_lot::Mutex;
pub use settings::*;
pub use voxel_generation::*;

#[derive(Global)]
// The global terrain component that can be added at the start of the game
pub struct Terrain {
    // Handler for our chunks
    pub(crate) chunks_manager: ChunksManager,
    // Handler for our voxel generation
    pub(crate) voxel_generator: VoxelGenerator,
    // Terrain edits manager
    pub(crate) editing_manager: EditingManager,
}

impl Terrain {
    // Create a new terrain global
    pub fn new(settings: TerrainSettings, pipeline: &Pipeline) -> Self {
        Self {
            chunks_manager: ChunksManager {
                octree: Arc::new(Mutex::new(DiffOctree::new(
                    settings.depth,
                    CHUNK_SIZE as u64,
                    settings.heuristic_settings,
                ))),
                material: settings.material,
                ..Default::default()
            },
            voxel_generator: VoxelGenerator::new(
                &settings.voxel_src_path,
                settings.uniforms,
                pipeline,
            ),
            editing_manager: EditingManager::default(),
        }
    }
    // Add a terrain edit
    pub fn edit(&mut self, edit: Edit) {
        self.editing_manager.edit(edit);
    }
    // Force the re-generation of a specific chunk
    pub fn regenerate_chunk(
        &mut self,
        coords: ChunkCoords,
        camera_position: veclib::Vector3<f32>,
        camera_forward: veclib::Vector3<f32>,
    ) -> Option<()> {
        // Check if the chunk is valid first
        if self.chunks_manager.chunks.contains_key(&coords) {
            // Regenerate
            if self.chunks_manager.chunks_generating.insert(coords) {
                // First time we queue this chunk for generation
                let id = self.chunks_manager.chunks.get(&coords)?;
                let priority = crate::components::Chunk::calculate_priority(
                    coords,
                    camera_position,
                    camera_forward,
                );
                self.chunks_manager.priority_list.push((*id, priority));
            } else {
                // Already queued for generation
            }
            Some(())
        } else {
            // The chunk does not exist yet
            return None;
        }
    }
}
