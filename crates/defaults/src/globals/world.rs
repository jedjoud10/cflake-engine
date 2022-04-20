use world::{ecs::Entity, globals::Global};

// Some global world data
#[derive(Default, Global)]
pub struct GlobalWorldData {
    // The main camera entity
    pub camera: Entity,
}
