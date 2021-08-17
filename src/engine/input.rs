use std::{
    collections::HashMap,
    fmt::{self},
};
extern crate glfw;
use glfw::{Action, Key};

// Status of a map
#[derive(Clone, Copy, Debug)]
pub enum MapStatus {
    Released,  // The frame the key was released on
    Held(f32), // If the key was held more than a frame
    Pressed,   // The frame the key was pressed on
    Nothing,   // If nothing happens in that specific frame
}

impl fmt::Display for MapStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// A simple input manager that reads keys from the keyboard and binds them to specific mappings
// Key -> Map
#[derive(Default)]
pub struct InputManager {
    pub bindings: HashMap<Key, String>,
    pub keys: HashMap<Key, MapStatus>,
    pub mappings: HashMap<String, MapStatus>,
    last_mouse_pos: (i32, i32),
	last_mouse_scroll: f32
}

impl InputManager {
    // Setup the default input bindings
    pub fn setup_default_bindings(&mut self) {
        self.bind_key(Key::Escape, "quit");
        self.bind_key(Key::F1, "fullscreen");
        self.bind_key(Key::F2, "capture_fps");
        self.bind_key(Key::F3, "change_debug_view");
		self.bind_key(Key::F, "toggle_wireframe")
    }
    // Called at the start of every frame to handle default-like events, like quitting by pressing Escape or fullscreening by pressing F1
    pub fn update(&mut self, _window: &mut glfw::Window) {
        // Update mappings first
        self.update_mappings();
        // Calculate the mouse delta
        
    }
	// Called when we recieve a new mouse event from the window (Could either be a mouse position one or a scroll one)
	pub fn recieve_mouse_event(&mut self, position: Option<(f64, f64)>, scroll: Option<f64>) {
		match position {
    		Some(position) => {
				// This is a mouse position event
				let mouse_pos = (
					position.0 as i32,
					position.1 as i32,
				);
				self.last_mouse_pos = mouse_pos;
			},
			_ => {}
		}
		match scroll {
    		Some(scroll) => {
				// This is a mouse scroll event
				self.last_mouse_scroll += scroll as f32;
			},
    		_ => {}
		}
	}
    // Update event fired from the world (fired after everything happens)
    pub fn late_update(&mut self, delta_time: f32) {
        for key in self.keys.iter_mut() {
            match key.1 {
                MapStatus::Released => {
                    // Go from "Released" to "Nothing"
                    *key.1 = MapStatus::Nothing;
                }
                MapStatus::Held(old_time) => {
                    // Add delta time to the held seconds counter
                    *key.1 = MapStatus::Held(*old_time + delta_time);
                }
                MapStatus::Pressed => {
                    // Go from "Pressed" to "Held"
                    *key.1 = MapStatus::Held(0.0);
                }
                MapStatus::Nothing => {}
            }
        }
    }
    // Get the accumulated mouse position
    pub fn get_accumulated_mouse_position(&self) -> (i32, i32) {
        self.last_mouse_pos
    }
	// Get the accumulated mouse scroll
	pub fn get_accumulated_mouse_scroll(&self) -> f32 {
		self.last_mouse_scroll
	}
    // Update the maps using the keys
    fn update_mappings(&mut self) {
        // Update the mappings
        for binding in self.bindings.iter() {
            // Make sure the keys already in the dictionary
            if self.keys.contains_key(binding.0) {
                // First of all, get the map status from they keys, since it talks directly to GLFW
                let new_mapstatus = self.keys[binding.0];
                // Then, update the internal mappings so it uses this new map-status
                *self.mappings.get_mut(binding.1).unwrap() = new_mapstatus;
            }
        }
    }
    // When we receive a key event from glfw (Always at the start of the frame)
    pub fn receive_key_event(&mut self, key: Key, action_type: Action) {
        // If this key does not exist in the dictionary yet, insert it
        self.keys.entry(key).or_insert(MapStatus::Nothing);
        match action_type {
            Action::Release => {
                *self.keys.get_mut(&key).unwrap() = MapStatus::Released;
            }
            Action::Press => {
                *self.keys.get_mut(&key).unwrap() = MapStatus::Pressed;
            }
            _ => {}
        }
    }
    // Binds a key to a specific mapping
    pub fn bind_key(&mut self, key: Key, map_name: &str) {
        // Check if the binding exists
        if let std::collections::hash_map::Entry::Vacant(e) = self.bindings.entry(key) {
            // The binding does not exist yet, so create a new one
            e.insert(map_name.to_string());
            self.mappings
                .insert(map_name.to_string(), MapStatus::Nothing);
        } else {
            // Nein.
        }
    }
    // Returns true when the map is pressed
    pub fn map_pressed(&self, name: &str) -> bool {
        // Make sure that mapping actually exists
        if self.mappings.contains_key(&name.to_string()) {
            match self.mappings[&name.to_string()] {
                MapStatus::Pressed => true,
                _ => false,
            }
        } else {
            false
        }
    }
    // Returns true when the map is being held
    pub fn map_held(&self, name: &str) -> (bool, f32) {
        // Make sure that mapping actually exists
        if self.mappings.contains_key(&name.to_string()) {
            match self.mappings[&name.to_string()] {
                MapStatus::Held(held_seconds) => (true, held_seconds),
                _ => (false, 0.0),
            }
        } else {
            (false, 0.0)
        }
    }
    // Returns true when the map has been released
    pub fn map_released(&self, name: &str) -> bool {
        if self.mappings.contains_key(&name.to_string()) {
            match self.mappings[&name.to_string()] {
                MapStatus::Released => true,
                _ => false,
            }
        } else {
            false
        }
    }
    // Gets the status of a specific map
    pub fn get_map_status(&self, name: String) -> MapStatus {
        if self.mappings.contains_key(&name) {
            self.mappings[&name]
        } else {
            MapStatus::Nothing
        }
    }
}
