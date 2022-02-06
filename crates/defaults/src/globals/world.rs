use main::globals::Global;

// Some global world data
#[derive(Default, Global)]
pub struct GlobalWorldData {
    // Camera values
    pub camera_pos: veclib::Vector3<f32>,
    pub camera_dir: veclib::Vector3<f32>,
    // World values
    pub sun_dir: veclib::Vector3<f32>,
}
