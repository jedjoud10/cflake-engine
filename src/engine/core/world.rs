use std::collections::HashMap;

use crate::engine::core::ecs::*;
use crate::engine::input::*;
use crate::engine::rendering::*;
use crate::engine::resources::Resource;
use crate::engine::resources::ResourceManager;
use crate::game::level::*;


//  The actual world
pub struct World {
	pub time_manager: Time,
	pub component_manager: ComponentManager,
	pub input_manager: InputManager,
	pub resource_manager: ResourceManager,
	pub shader_manager: ShaderManager,
	pub entity_manager: EntityManager,
	pub systems: Vec<Box<System>>,
} 

// Default world values
impl Default for World {
	fn default() -> Self {
		Self {			
			component_manager: ComponentManager { current_component_id: 1, ..ComponentManager::default() },
			time_manager: Time::default(),
			input_manager: InputManager::default(),
			resource_manager: ResourceManager::default(),
			shader_manager: ShaderManager::default(),
			entity_manager: EntityManager::default(),
			systems: Vec::new(),
		}
	}
}

impl World {
	// When the world started initializing
 	pub fn start_world(&mut self) {
		// Load all the default things
		self.input_manager.setup_default_bindings();
		register_components(self);
		load_systems(self);
		load_entities(self);
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

		// and render them
		self.run_entity_loop_on_system_type(SystemType::Render);
		

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
		let mut clone = self.systems.clone();
		for system in clone.iter_mut().filter(|sys| 			
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
		self.systems = clone;
	}
 	// When we want to close the application
	pub fn stop_world(&mut self) {
		let mut clone = self.systems.clone();
		for system in clone.iter_mut() {
			system.system_data.end_system(self);
		}
		self.systems = clone;
	}	
	// Check if a specified entity fits the criteria to be in a specific system
	fn is_entity_valid_for_system(entity: &Entity, system_data: &mut SystemData) -> bool {
		// Check if the system matches the component ID of the entity
		entity.c_bitfield >= system_data.c_bitfield
	}		
	// Adds a system to the world
	pub fn add_system(&mut self, mut system: Box<System>) {
		let mut system_data = &mut system.system_data;
		system_data.system_addded();
		println!("Add system with cBitfield: {}", system_data.c_bitfield);
		self.systems.push(system);
	}
	// Wrapper function around the entity manager's add_entity
	pub fn add_entity(&mut self, entity: Entity) -> u16 {
		let id = self.entity_manager.add_entity(entity.clone());
		let mut entity = self.entity_manager.get_entity(id).clone();
		// Check if there are systems that need this entity
		let mut clone = self.systems.clone();
		for system in clone.iter_mut() {
			let mut system_data = &mut system.system_data;
			if Self::is_entity_valid_for_system(&entity, system_data) {
				// Add the entity to the system
				system_data.add_entity(&entity, self);
			}		
		}
		// Since we cloned the entity variable we gotta update the entity manager with the new one
		*self.entity_manager.get_entity(id) = entity;
		self.systems = clone;
		return id;
	} 
	// Wrapper function around the entity manager remove_entity
	pub fn remove_entity(&mut self, entity_id: u16) {
		let removed_entity = self.entity_manager.remove_entity(entity_id);
		// Remove the entity from all the systems it was in
		let mut clone = self.systems.clone();
		for system in clone.iter_mut() {
			let mut system_data = &mut system.system_data;

			// Only remove the entity from the systems that it was in
			if removed_entity.c_bitfield >= system_data.c_bitfield {
				system_data.remove_entity(entity_id, &removed_entity, self);				
			}			
		}
		self.systems = clone;
	}
	//
	pub fn get_entity(&mut self, entity_id: u16) -> &mut Entity {
		self.entity_manager.get_entity(entity_id)
	}
}

// An entity manager that handles entities
#[derive(Default)]
pub struct EntityManager {
	pub entities: HashMap<u16, Entity>,
}

impl EntityManager {
	// Add an entity to the world 
	pub fn add_entity(&mut self, mut entity: Entity) -> u16 {
		entity.entity_id = self.entities.len() as u16;
		println!("\x1b[32mAdd entity '{}' with entity ID: {} and cBitfield: {}\x1b[0m", entity.name, entity.entity_id, entity.c_bitfield);		
		// Add the entity to the world
		let id = entity.entity_id;
		self.entities.insert(entity.entity_id, entity);
		return id;
	}
	// Get an entity using the entities vector and the "mapper (WIP)"
	pub fn get_entity(&mut self, entity_id: u16) -> &mut Entity {
		self.entities.get_mut(&entity_id).unwrap()
	}
	// Removes an entity from the world 
	pub fn remove_entity(&mut self, entity_id: u16) -> Entity {
		//println!("{:?}", self.entities);
		let removed_entity = self.entities.remove(&entity_id).expect("Entity does not exist, so it could not be removed!");
		println!("\x1b[33mRemove entity '{}' with entity ID: {} and cBitfield: {}\x1b[0m", removed_entity.name, removed_entity.entity_id, removed_entity.c_bitfield);	
		return removed_entity;
	}	
}

// Static time variables
#[derive(Default)]
pub struct Time {
	pub time_since_start: f64,
	pub delta_time: f64,
}