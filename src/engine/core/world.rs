use crate::engine::core::ecs::*;

//  The actual world
pub struct World {
	pub time_manager: Time,
	pub current_level: Level<System>,
} 
impl World {
	// When the world started initializing
 	pub fn start_world(&mut self) {
	}
	// When we want to draw a new frame onto the screen
 	pub fn update_world(&mut self) {		
	}
 	// When we want to close the application
	pub fn stop_world(&mut self) {
	}
}
 
// Default values for world
impl Default for World {
	fn default() -> Self {
	 	Self {
			// Setup the time manager
	 		time_manager: Time {
				time_since_start: 0.0,
	 			delta_time: 0.0,
			},

			current_level: 
	 	}
	} 
}

// A level that contains specific systems and entities
struct Level {
	entities: Vec<Entity>,
	systems: Vec<Box<dyn System>>,
}

// Implement level functions
impl Level {
	fn update_systems(&mut self) {
		// Update the systems
		for system in self.systems.iter_mut() {
			system.update_system()
		}
	}
}


// Static time variables
pub struct Time {
	pub time_since_start: f64,
	pub delta_time: f64,
}