use main::ecs::{impl_component, entity::EntityID};

// Some global world data
#[derive(Default)]
pub struct GlobalWorldData {
    // Camera values
    pub camera_pos: veclib::Vector3<f32>,

}

impl_component!(GlobalWorldData);