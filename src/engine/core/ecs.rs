use std::collections::HashMap;
use crate::engine::core::world::World;

// Maximum amount of components allowed on an entity
const MAX_COMPONENTS: u16 = 16;

// A component trait that can be added to other components
pub trait Component {
}

pub trait ComponentNames {
	fn get_component_name() -> String;
}

// Struct used to get the component ID of specific components, entities, and systems
pub struct ComponentID {
	pub components: HashMap<String, u16>,	
}

// Implement default values
impl Default for ComponentID {
	fn default() -> Self { 
		Self {
			components: HashMap::new(),
		}
	}
}

// Implement all the functions
impl ComponentID {
	// Get the component id for a specific entity
	pub fn get_component_id<T: ComponentNames>(&mut self) -> u16 {
		let name: String = T::get_component_name();
		// It found the component, so just return it's id
		if self.components.contains_key(&name) {
			let value = self.components[&name];
			return value;
		}
		
		// It did not find the component, so create a new one
		self.components.insert(name, self.components.len() as u16);
		self.components.len() as u16
	}

	// Get the component id for a specific entity
	pub fn get_component_id_by_name(&mut self, name: String) -> u16 {
		// It found the component, so just return it's id
		if self.components.contains_key(&name) {
			let value = self.components[&name];
			return value;
		}
		
		// It did not find the component, so create a new one
		self.components.insert(name, self.components.len() as u16);
		self.components.len() as u16
	}
}

// A system that can write/read component data, every frame, or at the start of the game
pub trait System {

	// Basic control code
	fn system_addded(&mut self);
	fn system_enabled(&mut self);
	fn system_disabled(&mut self);
	fn update_system(&mut self);
	fn add_component(&mut self, id: u16);
}

// A simple entity in the world
pub struct Entity {
	pub name: String,
	pub entity_id: usize,
	pub components_id: u16,
	components: HashMap<u16, Box<dyn Component>>,
}

// ECS time bois
impl Entity {
	// Link a component to this entity
	pub fn link_component<T: ComponentNames, U: Component + 'static>(&mut self, world: &mut World, component: U) {
		let component_id = world.component_manager.get_component_id::<T>();
		self.components_id = self.components_id | component_id;
		self.components.insert(component_id, Box::new(component));
	}
	// Unlink a component from this entity
	pub fn unlink_component<T: ComponentNames>(&mut self, world: &mut World) {
		let name = T::get_component_name();
		let id = world.component_manager.get_component_id_by_name(name);
		// Take the bit, invert it, then AND it to the bitfield
		self.components_id = (!id) & self.components_id;
		self.components.remove(&id);
	}
	// Gets a specific component
	pub fn get_component<T: ComponentNames, U: Component>(&mut self, world: &mut World) -> &Box<dyn Component> {
		let name = T::get_component_name();
		let id = world.component_manager.get_component_id_by_name(name);
		let value = &self.components[&id];
		value
	}
}

// Default
impl Default for Entity {
	fn default() -> Self {
		Self {
			name: String::from("Unnamed Entity"),
			entity_id: 0,
			components_id: 0,
			components: HashMap::new(),
		}
	}
}