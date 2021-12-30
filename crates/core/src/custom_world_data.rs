use ecs::EntityID;

// Some custom world data that is stored on the main thread, but can be shared around the threads
#[derive(Default, Clone)]
pub struct CustomWorldData {
    pub main_camera_entity_id: Option<EntityID>,
    pub light_dir: veclib::Vector3<f32>,
}
