use crate::engine::core::ecs::*;
use crate::game::levels::load_default_level;

//  The actual world
pub struct World {
	pub time_manager: Time,
	pub component_manager: ComponentManager,
	pub entities: Vec<Box<Entity>>,
	pub systems: Vec<Box<dyn System>>,
	pub update_systems: Vec<Box<dyn UpdateSystem>>,
	pub render_systems: Vec<Box<dyn RenderSystem>>,
	pub tick_systems: Vec<Box<dyn TickSystem>>,
} 
impl World {
	// When the world started initializing
 	pub fn start_world(&mut self) {
		load_default_level(self);
		unsafe {
			gl::ClearColor(0.0, 0.0, 0.0, 1.0);
		}
	}
	// We do the following in this function
	// 1. We update the entities of each UpdateSystem
	// 2. We tick the entities of each TickSystem (Only if the framecount is valid)
	// 3. We render the entities onto the screen using the RenderSystem
 	pub fn update_world(&mut self) {
		// update the entities, then render them
		for update_system in self.update_systems.iter_mut() {
			// Get the main system trait
			let system = self.systems.get(update_system.get_system_id() as usize).unwrap();	
			
			// Update the entities
			for entity in system.get_system_data().entities {
				update_system.update_entity(self.get_entity(entity));
			}
		}		

		unsafe {
			gl::Clear(gl::COLOR_BUFFER_BIT);
		}

	}
 	// When we want to close the application
	pub fn stop_world(&mut self) {
	}
	// Add an entity to the world 
	pub fn add_entity(&mut self, mut entity: Box<Entity>) {
		entity.entity_id = self.entities.len() as u16;
		println!("Add entity '{}' with entityid: {} and componentbitfieldid: {}", entity.name, entity.entity_id, entity.components_bitfield);

		//Check if there are any systems that could use this entity
		for system in self.systems.iter_mut() {
			// Check if the system matches the component ID of the entity
			if entity.components_bitfield >= system.get_system_data().component_bitfield {
				system.get_system_data().add_entity(entity.entity_id as u16, self);
			}
		}

		// Add the entity to the world
		self.entities.push(entity);
	}
	// Removes an entity from the world 
	pub fn remove_entity(&mut self, entity_id: u16) {
		self.entities.remove(entity_id as usize);
	}
	// Adds a system to the world and enables it 
	pub fn add_system(&mut self, mut system: Box<System>) {
		let mut system_data = system.get_system_data();
		system_data.system_addded();
		system_data.enable_system();
		println!("Add system with componentbitfieldid: {}", system_data.component_bitfield);
		self.systems.push(system);
	}
	// Get an entity using the entities vector
	pub fn get_entity(&self, entity_id: u16) -> &Box<Entity> {
		self.entities.get(entity_id as usize).unwrap()
	}
}

// Default values for world
impl Default for World {
	fn default() -> Self {		
	 	Self {
			// Setup the time manager
	 		time_manager: Time::default(),
			component_manager: ComponentManager::default(),
			entities: Vec::new(),
			systems: Vec::new(),
			render_systems: Vec::new(),
			tick_systems: Vec::new(),
			update_systems: Vec::new()
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