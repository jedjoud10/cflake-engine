use std::cmp::Ordering;

use ahash::{AHashMap, AHashSet};
use enum_as_inner::EnumAsInner;

use getset::Getters;
use world::{
    ecs::Entity,
    math::octrees::DiffOctree,
    rendering::{basics::material::Material, pipeline::Handle},
    terrain::{ChunkCoords, VoxelDataBufferId},
};

// Generation state of the current chunk
#[derive(EnumAsInner, PartialEq)]
pub enum ChunkGenerationState {
    RequiresVoxelData,
    FetchShaderStorages(Entity, ChunkCoords),
    EndVoxelDataGeneration(Entity, bool, Option<VoxelDataBufferId>),
}

impl Default for ChunkGenerationState {
    fn default() -> Self {
        Self::RequiresVoxelData
    }
}

#[derive(Getters, Default)]
#[getset(get = "pub")]
pub struct ChunksManager {
    // Chunk generation
    pub(crate) octree: DiffOctree,
    pub(crate) chunks: AHashMap<ChunkCoords, Entity>,
    pub(crate) chunks_generating: AHashSet<ChunkCoords>,
    pub(crate) priority_list: Vec<(Entity, f32)>,
    pub(crate) chunks_to_remove: Vec<Entity>,
    pub(crate) material: Handle<Material>,
    pub(crate) physics: bool,
    pub(crate) must_update_octree: bool,

    // The Entity ID of the chunk that we are generating
    // This includes voxel data generation AND mesh generation
    pub(crate) current_chunk_state: ChunkGenerationState,
}

impl ChunksManager {
    // Update the priority list
    pub fn update_priorities(&mut self) {
        self.priority_list.sort_by(|(_, x), (_, y)| f32::partial_cmp(x, y).unwrap_or(Ordering::Equal));
    }
}
