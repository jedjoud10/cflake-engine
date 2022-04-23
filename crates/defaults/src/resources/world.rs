use world::{ecs::Entity, resources::Resource};

// Some global world data
#[derive(Default, Resource)]
pub struct WorldData {
    // The main camera entity
    pub camera: Entity,
}
