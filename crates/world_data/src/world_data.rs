use std::rc::Rc;
use rendering::basics::Texture;
use rendering::utils::Window;

// Data that will be passed to the fire events in systems
pub struct WorldData<'a> {
    pub entity_manager: &'a mut ecs::EntityManager,
    pub component_manager: &'a mut ecs::ComponentManager,
    pub ui_manager: &'a mut ui::UIManager,
    pub input_manager: &'a mut input::InputManager,
    pub asset_manager: &'a mut assets::AssetManager,
    pub time_manager: &'a mut others::Time,
    pub debug: &'a mut debug::MainDebug,
    pub custom_data: &'a mut CustomWorldData,
    pub instance_manager: &'a mut others::InstanceManager,
}
// Some custom data that will be passed to systems
#[derive(Default)]
pub struct CustomWorldData {
    pub main_camera_entity_id: usize,
    pub sky_texture: Option<Rc<Texture>>,
    pub render_system_id: u8,
    pub window: Window,
    pub light_dir: veclib::Vector3<f32>,
}
