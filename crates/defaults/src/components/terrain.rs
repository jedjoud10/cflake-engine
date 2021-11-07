use assets::AssetManager;
use ecs::{Component, ComponentID, ComponentInternal};
use terrain::{Terrain, TerrainSettings};

// Terrain data that will be on the terrain entity
pub struct TerrainData {
    pub terrain: Terrain,
}

impl TerrainData {
    pub fn new(settings: TerrainSettings, asset_manager: &mut AssetManager) -> Self {
        Self {
            terrain: Terrain::new(settings, asset_manager)
        }
    }
}

ecs::impl_component!(TerrainData);
