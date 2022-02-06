use std::collections::{HashMap, HashSet};

use main::{
    ecs::entity::EntityID,
    math::octrees::DiffOctree,
    rendering::{basics::material::Material, object::ObjectID},
    terrain::ChunkCoords,
};

#[derive(Default)]
pub struct ChunksHandler {
    // Chunk generation
    pub octree: DiffOctree,
    pub chunks: HashMap<ChunkCoords, EntityID>,
    pub chunks_generating: HashSet<ChunkCoords>,
    pub sorted_chunks_generating: Vec<(EntityID, f32)>,
    pub chunks_to_remove: Vec<EntityID>,
    pub material: ObjectID<Material>,

    // The Entity ID of the chunk that we are generating this voxel data for
    pub chunk_id: Option<EntityID>,
    // We also store the Entity ID of the chunk whom we must create the mesh for
    pub mesh_gen_chunk_id: Option<EntityID>,
}

impl ChunksHandler {
    // Create a new chunks handler using some default values
    pub fn new(octree: DiffOctree) -> Self {
        Self { octree, ..Default::default() }
    }
}
