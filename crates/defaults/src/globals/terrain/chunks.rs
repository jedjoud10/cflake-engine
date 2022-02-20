use std::collections::{HashMap, HashSet};

use enum_as_inner::EnumAsInner;
use main::{
    ecs::entity::EntityID,
    math::octrees::DiffOctree,
    rendering::{basics::material::Material, object::ObjectID},
    terrain::ChunkCoords,
};
// Generation state of the current chunk
#[derive(EnumAsInner, Debug, PartialEq)]
pub enum ChunkGenerationState {
    RequiresVoxelData,
    BeginVoxelDataGeneration(EntityID),
    EndVoxelDataGeneration(EntityID, bool),
}

impl Default for ChunkGenerationState {
    fn default() -> Self {
        Self::RequiresVoxelData
    }
}

#[derive(Default)]
pub struct ChunksHandler {
    // Chunk generation
    pub octree: DiffOctree,
    pub chunks: HashMap<ChunkCoords, EntityID>,
    pub chunks_generating: HashSet<ChunkCoords>,
    pub sorted_chunks_generating: Vec<(EntityID, f32)>,
    pub chunks_to_remove: Vec<EntityID>,
    pub material: ObjectID<Material>,

    // The Entity ID of the chunk that we are generating
    // This includes voxel data generation AND mesh generation
    pub current_chunk_state: ChunkGenerationState,
}
