use crate::engine::core::ecs::*;

// Camera system
pub struct CameraSystem {
	pub system_data: SystemData	
}

impl System for CameraSystem {
    fn get_system_data(&self) -> SystemData {
        self.system_data
    }

    fn set_system_data(&self, system: SystemData) {
        todo!()
    }
}

// Update system trait for camera system
impl UpdateSystem for CameraSystem {
    fn update_entity(&self, entity: &Box<Entity>) {
        
    }

	// Get the world ID of this system
    fn get_system_id(&self) -> u8 {
        self.system_data.system_id
    }
}

// Render system trait for camera system
impl RenderSystem for CameraSystem {
    fn render_entity(&self, entity: &Box<Entity>) {
        
    }

	// Get the world ID of this system
    fn get_system_id(&self) -> u8 {
        self.system_data.system_id
    }
}