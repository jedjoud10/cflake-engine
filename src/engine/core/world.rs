use crate::engine::core::ecs::*;
use crate::engine::core::input::*;
use crate::game::level::*;


//  The actual world
pub struct World {
	pub time_manager: Time,
	pub component_manager: ComponentManager,
	pub input_manager: InputManager,
	entities: Vec<Box<Entity>>,
	systems: Vec<Box<System>>,
} 
impl World {
	// When the world started initializing
 	pub fn start_world(&mut self) {
		// Load all the default things
		self.input_manager.setup_default_bindings();
		register_components(self);
		load_systems(self);
		load_entities(self);
		unsafe {
			gl::ClearColor(0.0, 0.0, 0.0, 1.0);
		}
	}
	// We do the following in this function
	// 1. We update the entities of each UpdateSystem
	// 2. We tick the entities of each TickSystem (Only if the framecount is valid)
	// 3. We render the entities onto the screen using the RenderSystem
 	pub fn update_world(&mut self, window: &mut glfw::Window) {
		// Check for input events
		self.input_manager.update(window);
		// Update the entities
		self.run_entity_loop_on_system_type(SystemType::Update);

		// Render the entities
		unsafe {
			// Clear the screen first
			gl::Clear(gl::COLOR_BUFFER_BIT);
			self.run_entity_loop_on_system_type(SystemType::Render);
		}

		// Update the up-time of every system
		for system in self.systems.iter_mut() {
			match system.system_data.state {
    			SystemState::Enabled(time) => { system.system_data.state = SystemState::Enabled(time + self.time_manager.delta_time as f32); },
    			SystemState::Disabled(time) => { system.system_data.state = SystemState::Disabled(time + self.time_manager.delta_time as f32); },
			}
		}
 
		// Update the inputs
		self.input_manager.late_update(self.time_manager.delta_time as f32);
	}
	// Triggers the "run_entity_loop" event on a specific type of system
	fn run_entity_loop_on_system_type(&mut self, system_type: SystemType) {
		for system in self.systems.clone().iter_mut().filter(|sys| 			
			match &sys.system_data.stype {
				system_type => true,
				_ => false
		} ) {
			match &system.system_data.state {
    			SystemState::Enabled(_) => {
					system.system_data.run_entity_loops(self);
				},
    			_ => {	}
			}
		}	
	}
 	// When we want to close the application
	pub fn stop_world(&mut self) {
		for system in self.systems.clone().iter_mut() {
			system.system_data.end_system(self);
		}
	}
	// Add an entity to the world 
	pub fn add_entity(&mut self, mut entity: Box<Entity>) {
		entity.entity_id = self.entities.len() as u16;
		println!("Add entity '{}' with entity ID: {} and cBitfield: {}", entity.name, entity.entity_id, entity.c_bitfield);

		// Check if there are systems that need this entity
		for system in self.systems.clone().iter_mut() {
			let mut system_data = &mut system.system_data;
			if Self::is_entity_valid_for_system(&entity, system_data) {
				let clone = entity.clone();
				// Add the entity to the update system
				system_data.add_entity(clone, self);
			}		
		}
		// Add the entity to the world
		self.entities.push(entity);
	}
	// Check if a specified entity fits the criteria to be in a specific system
	fn is_entity_valid_for_system(entity: &Box<Entity>, system_data: &mut SystemData) -> bool {
		// Check if the system matches the component ID of the entity
		entity.c_bitfield >= system_data.c_bitfield
	}
	// Removes an entity from the world 
	pub fn remove_entity(&mut self, entity_id: u16) {
		let removed_entity = self.entities.remove(entity_id as usize);
		println!("Remove entity '{}' with entity ID: {} and cBitfield: {}", removed_entity.name, removed_entity.entity_id, removed_entity.c_bitfield);

		// Remove the entity from all the systems it was in
		for system in self.systems.clone().iter_mut() {
			let mut system_data = &mut system.system_data;

			// Only remove the entity from the systems that it was in
			if removed_entity.c_bitfield >= system_data.c_bitfield {
				system_data.remove_entity(entity_id, self);				
			}			
		}
	}	
	// Adds a system to the world
	pub fn add_system(&mut self, mut system: Box<System>) {
		let mut system_data = &mut system.system_data;
		system_data.system_addded();
		system_data.enable_system();
		println!("Add system with cBitfield: {}", system_data.c_bitfield);
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
			input_manager: InputManager::default(),
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