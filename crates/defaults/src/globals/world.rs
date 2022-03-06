use world::{ecs::entity::EntityKey, globals::Global};

// Some global world data
#[derive(Default, Global)]
pub struct GlobalWorldData {
    // Camera values
    pub camera_pos: veclib::Vector3<f32>,
    pub camera_forward: veclib::Vector3<f32>,
    pub camera_right: veclib::Vector3<f32>,
    pub camera_up: veclib::Vector3<f32>,
    pub camera_entity_key: EntityKey,
}
