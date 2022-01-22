use std::collections::HashMap;
use main::{terrain::ChunkCoords, ecs::{entity::EntityID, impl_component}, math, rendering::{ object::ObjectID, basics::texture::Texture, advanced::compute::ComputeShader}};


// A terrain component that can be added to a terrain entity
pub struct Terrain {
    // Chunk generation
    pub octree: math::octrees::AdvancedOctree,
    pub chunks: HashMap<ChunkCoords, EntityID>,

    // Voxel Generation
    pub compute_shader: ObjectID<ComputeShader>,
    pub voxel_texture: ObjectID<Texture>,
    pub material_texture: ObjectID<Texture>,
}

impl_component!(Terrain);