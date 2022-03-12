use std::cmp::Ordering;

use ahash::{AHashMap, AHashSet};
use enum_as_inner::EnumAsInner;

use world::{
    ecs::entity::EntityKey,
    math::octrees::DiffOctree,
    rendering::{basics::material::Material, pipeline::Handle},
    terrain::ChunkCoords,
};
// Generation state of the current chunk
#[derive(EnumAsInner, Debug, PartialEq)]
pub enum ChunkGenerationState {
    RequiresVoxelData,
    FetchShaderStorages(EntityKey, ChunkCoords),
    EndVoxelDataGeneration(EntityKey, bool),
}

impl Default for ChunkGenerationState {
    fn default() -> Self {
        Self::RequiresVoxelData
    }
}

#[derive(Default)]
pub struct ChunksManager {
    // Chunk generation
    pub octree: DiffOctree,
    pub chunks: AHashMap<ChunkCoords, EntityKey>,
    pub chunks_generating: AHashSet<ChunkCoords>,
    pub priority_list: Vec<(EntityKey, f32)>,
    pub chunks_to_remove: Vec<EntityKey>,
    pub material: Handle<Material>,

    // The Entity ID of the chunk that we are generating
    // This includes voxel data generation AND mesh generation
    pub current_chunk_state: ChunkGenerationState,
}

impl ChunksManager {
    // Update the priority list
    pub fn update_priorities(&mut self) {
        self.priority_list.sort_by(|(_, x), (_, y)| f32::partial_cmp(x, y).unwrap_or(Ordering::Equal));
    }
}
