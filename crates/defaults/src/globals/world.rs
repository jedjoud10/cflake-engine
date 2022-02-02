use main::ecs::component::Component;

// Some global world data
#[derive(Default, Component)]
pub struct GlobalWorldData {
    // Camera values
    pub camera_pos: veclib::Vector3<f32>,
    pub camera_dir: veclib::Vector3<f32>,
}
