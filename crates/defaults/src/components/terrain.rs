use ecs::{Component, ComponentID, ComponentInternal};
use terrain::Terrain;

// Terrain data that will be on the terrain entity
pub struct TerrainData {
    pub terrain: Terrain,
}

ecs::impl_component!(TerrainData);
