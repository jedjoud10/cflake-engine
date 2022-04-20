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
    // Octree
    pub(crate) octree: DiffOctree,
    pub(crate) chunks: AHashMap<ChunkCoords, Entity>,

    // To Generate
    pub(crate) chunks_generating: AHashSet<ChunkCoords>,

    // To Remove
    pub(crate) chunks_to_remove: Vec<Entity>,

    // Current
    pub(crate) current_chunk_state: ChunkGenerationState,
    
    // Other
    pub(crate) material: Handle<Material>,
    pub(crate) priority_list: Vec<(Entity, f32)>,
}

impl ChunksManager {
    // Update the priority list
    pub fn update_priorities(&mut self) {
        self.priority_list.sort_by(|(_, x), (_, y)| f32::partial_cmp(x, y).unwrap_or(Ordering::Equal));
    }
}
