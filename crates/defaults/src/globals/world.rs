use world::{ecs::entity::EntityKey, globals::Global};

// Some global world data
#[derive(Default, Global)]
pub struct GlobalWorldData {
    // The main camera entity
    pub main_camera: EntityKey,
}
