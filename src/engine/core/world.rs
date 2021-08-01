use std::collections::HashMap;
use glfw::Context;

use crate::engine::core::ecs::*;
use crate::engine::input::*;
use crate::engine::rendering::*;

use crate::engine::resources::ResourceManager;
use crate::engine::core::defaults::components::components::Camera;
use crate::engine::rendering::Window;
use crate::game::level::*;



//  The actual world
pub struct World {
	pub time_manager: Time,
	pub component_manager: ComponentManager,
	pub input_manager: InputManager,
	pub resource_manager: ResourceManager,
	pub shader_manager: ShaderManager,
	pub entity_manager: EntityManager,
	pub systems: Vec<System>,
	pub window: Window,
	pub default_camera_id: u16
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
			default_camera_id: 0,
			window: Window::default()
		}
	}
}

impl World {
	// When the world started initializing
 	pub fn start_world(&mut self, window: &mut glfw::Window) {
		// Load all the default things
		self.input_manager.setup_default_bindings();
		self.window.size = Self::get_default_window_size();
		window.set_cursor_mode(glfw::CursorMode::Disabled);
		window.set_cursor_pos(0.0, 0.0);
		register_components(self);
		load_systems(self);
		load_entities(self);
	}
	// We do the following in this function
	// 1. We update the entities of each UpdateSystem
	// 2. We tick the entities of each TickSystem (Only if the framecount is valid)
	// 3. We render the entities onto the screen using the RenderSystem
 	pub fn update_world(&mut self, window: &mut glfw::Window, glfw: &mut glfw::Glfw) {
		// Check for input events
		self.input_manager.update(window);		
		// Check for default input events
		self.check_default_input_events(window, glfw);
		// Update the entities
		self.run_entity_loop_on_system_type(SystemType::Update);

		// And render them
		self.run_entity_loop_on_system_type(SystemType::Render);
		window.swap_buffers();
		

		// Update the up-time of every system
		for system in self.systems.iter_mut() {
			match system.state {
    			SystemState::Enabled(time) => { system.state = SystemState::Enabled(time + self.time_manager.delta_time as f32); },
    			SystemState::Disabled(time) => { system.state = SystemState::Disabled(time + self.time_manager.delta_time as f32); },
			}
		}
 
		// Update the inputs
		self.input_manager.late_update(self.time_manager.delta_time as f32);
	}
	// Check for default key map events
	fn check_default_input_events(&mut self, window: &mut glfw::Window, glfw: &mut glfw::Glfw) {
		// Check for default mapping events
		if self.input_manager.map_pressed(String::from("Quit")) {
			window.set_should_close(true);			
		}
		// Toggle the fullscreen
		if self.input_manager.map_pressed(String::from("Fullscreen")) {
			self.toggle_fullscreen(glfw, window);
		}
	}
	// Toggle fullscreen
	pub fn toggle_fullscreen(&mut self, glfw: &mut glfw::Glfw, window: &mut glfw::Window) {
		self.window.fullscreen = !self.window.fullscreen;
		if self.window.fullscreen {
			// Set the glfw window as a fullscreen window
			glfw.with_primary_monitor_mut(|_glfw2, monitor| {
				let videomode = monitor.unwrap().get_video_mode().unwrap();	
				window.set_monitor(glfw::WindowMode::FullScreen(monitor.unwrap()), 0, 0, videomode.width, videomode.height, None);
				unsafe {
					// Update the OpenGL viewport
					gl::Viewport(0, 0, videomode.width as i32, videomode.height as i32);
				}
			});
		} else {
			// Set the glfw window as a windowed window
			glfw.with_primary_monitor_mut(|_glfw2, monitor| {
				let _videomode = monitor.unwrap().get_video_mode().unwrap();	
				let default_window_size = Self::get_default_window_size();
				window.set_monitor(glfw::WindowMode::Windowed, 50, 50, default_window_size.0 as u32, default_window_size.1 as u32, None);
				unsafe {
					// Update the OpenGL viewport
					gl::Viewport(0, 0, default_window_size.0 as i32, default_window_size.1 as i32);
				}
			});
		}		
	}
	// Triggers the "run_entity_loop" event on a specific type of system
	fn run_entity_loop_on_system_type(&mut self, _system_type: SystemType) {
		let mut clone = self.systems.clone();
		for system in clone.iter_mut().filter(|sys| 			
			match &sys.stype {
				_system_type => true,
				_ => false
		} ) {
			match &system.state {
    			SystemState::Enabled(_) => {
					system.run_entity_loops(self);
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
			system.end_system(self);
		}
		self.systems = clone;
	}			
	// Adds a system to the world
	pub fn add_system(&mut self, mut system: System) {
		let system_data = &mut system;
		system_data.system_addded();
		println!("Add system with cBitfield: {}", system_data.c_bitfield);
		self.systems.push(system);
	}	
	// Add a discrete component to the world, that isn't linked to any entity
	pub fn add_discrete_component<'a, T: ComponentID + Component + 'static>(&mut self, component: T) -> u16 {
		// Make sure the component is registered first
		if !self.component_manager.is_component_registered::<T>() {
			self.component_manager.register_component::<T>();
		}
		// Add the component, and return it's id
		self.component_manager.discrete_components.push(Box::new(component));
		let index = self.component_manager.discrete_components.len() as u16 - 1;
		return index;
		
	}
	// Get a reference to a specific discrete component from the world, without the need of an entity
	pub fn get_dicrete_component<'a, T: ComponentID + Component + 'static>(&self, index: u16) -> &T {
		let component_any = self.component_manager.discrete_components.get(index as usize).unwrap().as_any();
		let component: &T = component_any.downcast_ref().unwrap();
		return component;
	}
}

// Impl block for the entity stuff
impl World {
	// Wrapper function around the entity manager's add_entity
	pub fn add_entity(&mut self, entity: Entity) -> u16 {
		let id = self.entity_manager.add_entity(entity.clone());
		let entity = self.entity_manager.get_entity(id).clone();
		// Check if there are systems that need this entity
		let mut clone = self.systems.clone();
		for system in clone.iter_mut() {
			let system = system;
			if Self::is_entity_valid_for_system(&entity, system) {
				// Add the entity to the system
				system.add_entity(&entity, self);
			}		
		}
		// Since we cloned the entity variable we gotta update the entity manager with the new one
		*self.entity_manager.get_entity_mut(id) = entity;
		self.systems = clone;
		return id;
	} 
	// Wrapper function around the entity manager remove_entity
	pub fn remove_entity(&mut self, entity_id: u16) {
		let removed_entity = self.entity_manager.remove_entity(entity_id);
		// Remove the entity from all the systems it was in
		let mut clone = self.systems.clone();
		for system in clone.iter_mut() {
			let system = system;
		
			// Only remove the entity from the systems that it was in
			if removed_entity.c_bitfield >= system.c_bitfield {
				system.remove_entity(entity_id, &removed_entity, self);				
			}			
		}
		self.systems = clone;
	}
	// Get a mutable reference to an entity from the entity manager
	pub fn get_entity_mut(&mut self, entity_id: u16) -> &mut Entity {
		self.entity_manager.get_entity_mut(entity_id)
	}
	pub fn get_entity(&self, entity_id: u16) -> &Entity {
		self.entity_manager.get_entity(entity_id)
	}
	// Check if a specified entity fits the criteria to be in a specific system
	fn is_entity_valid_for_system(entity: &Entity, system_data: &mut System) -> bool {
		// Check if the system matches the component ID of the entity
		let bitfield: u16 = system_data.c_bitfield & !entity.c_bitfield;
		// If the entity is valid, all the bits would be 0
		return bitfield == 0;
	}	
}

// Impl block related to the windowing / rendering stuff
impl World {
	// Get the default width and height of the starting window
	pub fn get_default_window_size() -> (i32, i32) {
		(1280, 720)
	}
	// When we resize the window
	pub fn resize_window_event(&mut self, size: (i32, i32)) {
		unsafe {
			gl::Viewport(0, 0, size.0, size.1);
		}
		let camera_entity_clone = self.get_entity(self.default_camera_id).clone();
		let entity_clone_id = camera_entity_clone.entity_id;
		let camera_component = camera_entity_clone.get_component_mut::<Camera>(self);
		camera_component.aspect_ratio = size.0 as f32 / size.1 as f32;
		camera_component.window_size = size;
		camera_component.update_projection_matrix();
		// Update the original entity
		*self.get_entity_mut(entity_clone_id) = camera_entity_clone;
		self.window.size = size;
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
	// Get a mutable reference to a stored entity
	pub fn get_entity_mut(&mut self, entity_id: u16) -> &mut Entity {
		self.entities.get_mut(&entity_id).unwrap()
	}
	// Get an entity using the entities vector and the "mapper (WIP)"
	pub fn get_entity(&self, entity_id: u16) -> &Entity {
		self.entities.get(&entity_id).unwrap()
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