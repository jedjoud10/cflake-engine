use std::{collections::HashMap, hash::Hash};
extern crate glfw;
use glfw::{Key, Action};
use crate::engine::core::world::*;

// Status of a map
#[derive(Clone, Copy)]
pub enum MapStatus {
	Released, // The frame the key was released on
	Held(f32), // If the key was held more than a frame
	Pressed, // The frame the key was pressed on
	Nothing, // If nothing happens in that specific frame
}

// A simple input manager that reads keys from the keyboard and binds them to specific mappings
// Key -> Map  
pub struct InputManager {
	pub bindings: HashMap<String, Key>,
	pub maps: HashMap<String, MapStatus>,
	pub current_keys: HashMap<Key, MapStatus>,
}

impl InputManager {
	// Called at the start of every frame to handle default-like events, like quitting by pressing Escape or fullscreening by pressing F1
	pub fn default_input_event(&self) {
		if self.map_pressed(String::from("Quit")) {
			
		}

		if self.map_pressed(String::from("Fullscreen")) {
			
		}
	}
	// Update event fired from the world (fired after everything happens)
	pub fn update(&mut self, delta_time: f32) {
		for key in self.current_keys.iter_mut() {
			match key.1 {
    			MapStatus::Released => {
					// Go from "Released" to "Nothing"
					*key.1 = MapStatus::Nothing;
				},
    			MapStatus::Held(time) => {
					// Add delta time to the held seconds counter
					*key.1 = MapStatus::Held(delta_time);
				},
    			MapStatus::Pressed => {
					// Go from "Pressed" to "Held"
					*key.1 = MapStatus::Held(0.0);
				},
    			MapStatus::Nothing => {},
			}
		}
	}
	// When we receive a key even from glfw
	pub fn receive_key_event(&mut self, key: Key, action_type: Action) {
		match action_type {
    		Action::Release => {
				*self.current_keys.get_mut(&key).unwrap() = MapStatus::Released;
			},
    		Action::Press => {
				*self.current_keys.get_mut(&key).unwrap() = MapStatus::Pressed;
			},
			_ => {},
		}
	}
	// Bind a default key to a specific map
	pub fn bind_default_key_map(&mut self, key: Key, map: String) {
		// The map exists
		if self.bindings.contains_key(&map) {
			*self.bindings.get_mut(&map).unwrap() = key;
		} else {
			// The map does not exist yet
			self.bindings.insert(map, key);
		}
	}
	// Returns true when the map is pressed
	pub fn map_pressed(&self, name: String) -> bool {
		match self.maps[&name] {
			MapStatus::Pressed => true,
			_ => false,
		}
	}
	// Returns true when the map is being held
	pub fn map_held(&self, name: String) -> (bool, f32) {
		match self.maps[&name] {
			MapStatus::Held(held_seconds) => (true, held_seconds),
			_ => (false, 0.0),
		}
	}
	// Returns true when the map has been released
	pub fn map_released(&self, name: String) -> bool {
		match self.maps[&name] {
			MapStatus::Released => true,
			_ => false,
		}
	}
	// Gets the status of a specific map
	pub fn get_map_status(&self, name: String) -> MapStatus {
		self.maps[&name]
	}
}

impl Default for InputManager {
	fn default() -> Self {
		Self {
			bindings: HashMap::new(),
			maps: HashMap::new(),
			current_keys: HashMap::new(),
		}
	}
}