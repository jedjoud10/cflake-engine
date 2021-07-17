use crate::engine::core::ecs::*;
use crate::game::levels::load_default_level;

//  The actual world
pub struct World {
	pub time_manager: Time,
	pub component_manager: ComponentID,
	pub entities: Vec<Entity>,
	pub systems: Vec<Box<dyn System>>,
} 
impl World {
	// When the world started initializing
 	pub fn start_world(&mut self) {
		load_default_level(self);
	}
	// When we want to draw a new frame onto the screen
 	pub fn update_world(&mut self) {		
	}
 	// When we want to close the application
	pub fn stop_world(&mut self) {
	}
	// Add an entity to the world 
	pub fn add_entity(&mut self, mut entity: Entity) {
		entity.entity_id = self.entities.len();
		self.entities.push(entity);
	}
	// Removes an entity from the world 
	pub fn remove_entity(&mut self, entity: Entity) {
		self.entities.remove(entity.entity_id);
	}
	// Adds a system to the world and enables it 
	pub fn add_system<T>(&mut self, mut system: impl System + 'static) {
		system.system_addded();
		system.system_enabled();
		self.systems.push(Box::new(system));

		// TODO: Fix this sheit
	}
}

// Default values for world
impl Default for World {
	fn default() -> Self {		
	 	Self {
			// Setup the time manager
	 		time_manager: Time::default(),
			component_manager: ComponentID::default(),
			entities: Vec::new(),
			systems: Vec::new(),
	 	}
	} 
}

// Static time variables
pub struct Time {
	pub time_since_start: f64,
	pub delta_time: f64,
}

// Default
impl Default for Time {
	fn default() -> Self {		
		Self {
			time_since_start: 0.0,
			delta_time: 0.0
		}
   } 
}