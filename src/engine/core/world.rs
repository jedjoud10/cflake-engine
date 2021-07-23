use crate::engine::core::ecs::*;
use crate::game::levels::load_default_level;

//  The actual world
pub struct World {
	pub time_manager: Time,
	pub component_manager: ComponentManager,
	pub entities: Vec<Box<Entity>>,
	pub systems: Vec<Box<System>>,
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
		for update_system in self.systems.iter_mut().filter(|sys| 
			match sys.system_data.stype {
				SystemType::Update => true,
				_ => false
		} ) {
			
			// Update the entities
			for entity in (&update_system.system_data).entities.iter() {
				(update_system.call_entity_event)(&entity);
			}
		}		

		unsafe {
			// Clear the screen first
			gl::Clear(gl::COLOR_BUFFER_BIT);
			for render_system in self.systems.iter_mut().filter(|sys| 
				match sys.system_data.stype {
					SystemType::Render => true,
					_ => false
			} ) { 
				// Render each entity in the system
				for entity in (&render_system.system_data).entities.iter() {
					(render_system.call_entity_event)(&entity);
				}
			}
		}

	}
 	// When we want to close the application
	pub fn stop_world(&mut self) {
	}
	// Add an entity to the world 
	pub fn add_entity(&mut self, mut entity: Box<Entity>) {
		entity.entity_id = self.entities.len() as u16;
		println!("Add entity '{}' with entity ID: {} and cBitfield: {}", entity.name, entity.entity_id, entity.components_bitfield);

		// Check if there are systems that need this entity
		for system in self.systems.iter_mut() {
			let mut system_data = &mut system.system_data;
			if Self::is_entity_valid_for_system(&entity, system_data) {
				let clone = entity.clone();
				// Add the entity to the update system
				system_data.add_entity(clone);
			}		
		}
		// Add the entity to the world
		self.entities.push(entity);
	}
	// Check if a specified entity fits the criteria to be in a specific system
	fn is_entity_valid_for_system(entity: &Box<Entity>, system_data: &mut SystemData) -> bool {
		// Check if the system matches the component ID of the entity
		if entity.components_bitfield >= system_data.component_bitfield {		
			let pointer_copy = entity.clone();		
			system_data.add_entity(pointer_copy);
		}
		false
	}
	// Removes an entity from the world 
	pub fn remove_entity(&mut self, entity_id: u16) {
		let removed_entity = self.entities.remove(entity_id as usize);
		println!("Remove entity '{}' with entity ID: {} and cBitfield: {}", removed_entity.name, removed_entity.entity_id, removed_entity.components_bitfield);

		// Remove the entity from all the systems it was in
		for system in self.systems.iter_mut() {
			let mut system_data = &mut system.system_data;
			// Search for the entity with the matching entity_id
			let index = system_data.entities.iter_mut().position(|entity| entity.entity_id == entity_id).unwrap();
			system_data.entities.remove(index);	
		}
	}
	
	// Adds a system to the world
	pub fn add_system(&mut self, mut system: Box<System>) {
		let mut system_data = &mut system.system_data;
		system_data.system_addded();
		system_data.enable_system();
		println!("Add system with cBitfield: {}", system_data.component_bitfield);
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