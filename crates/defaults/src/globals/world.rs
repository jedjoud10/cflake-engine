use main::{globals::Global, ecs::entity::EntityID};

// Some global world data
#[derive(Default, Global)]
pub struct GlobalWorldData {
    // Camera values
    pub camera_pos: veclib::Vector3<f32>,
    pub camera_forward: veclib::Vector3<f32>,
    pub camera_right: veclib::Vector3<f32>,
    pub camera_up: veclib::Vector3<f32>,
    pub camera_entity_id: Option<EntityID>,
    // World values
    pub sun_quat: veclib::Quaternion<f32>,
}
