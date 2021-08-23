use std::{
    collections::HashMap,
    fmt::{self},
};

// Map data
#[derive(Clone, Copy, Debug)]
pub struct MapData {
    pub status: MapStatus,
    pub map_type: MapType,
}

impl Default for MapData {
    fn default() -> Self {
        Self {
            status: MapStatus::Nothing,
            map_type: MapType::Button,
        }
    }
}

// Status of a map
#[derive(Clone, Copy, Debug)]
pub enum MapStatus {
    Released,  // The frame the key was released on
    Held(f32), // If the key was held more than a frame
    Pressed,   // The frame the key was pressed on
    Nothing,   // If nothing happens in that specific frame
    ToggleOn,
    ToggleOff,
}

// The type of map
#[derive(Clone, Copy, Debug)]
pub enum MapType {
    Button, // You can press and release it, or even hold it
    Toggle // Just a toggle
}

// A simple input manager that reads keys from the keyboard and binds them to specific mappings
// Key -> Map
#[derive(Default)]
pub struct InputManager {
    pub bindings: HashMap<String, String>,
    pub keys: HashMap<String, MapData>,
    pub mappings: HashMap<String, MapData>,
    last_mouse_pos: (i32, i32),
    last_mouse_scroll: f32,
}

impl InputManager {
    // Setup the default input bindings
    pub fn setup_default_bindings(&mut self) {
        self.bind_key("", "quit", MapType::Button);
        self.bind_key("", "fullscreen", MapType::Button);
        self.bind_key("", "capture_fps", MapType::Button);
        self.bind_key("", "change_debug_view", MapType::Button);
        self.bind_key("", "toggle_wireframe", MapType::Button);
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
                let mouse_pos = (position.0 as i32, position.1 as i32);
                self.last_mouse_pos = mouse_pos;
            }
            _ => {}
        }
        match scroll {
            Some(scroll) => {
                // This is a mouse scroll event
                self.last_mouse_scroll += scroll as f32;
            }
            _ => {}
        }
    }
    // Update event fired from the world (fired after everything happens)
    pub fn late_update(&mut self, delta_time: f32) {
        for key in self.keys.iter_mut() {
            match key.1.status {
                MapStatus::Released => {
                    // Go from "Released" to "Nothing"
                    key.1.status = MapStatus::Nothing;
                }
                MapStatus::Held(old_time) => {
                    // Add delta time to the held seconds counter
                    key.1.status = MapStatus::Held(old_time + delta_time);
                }
                MapStatus::Pressed => {
                    // Go from "Pressed" to "Held"
                    key.1.status = MapStatus::Held(0.0);
                }
                MapStatus::ToggleOn => todo!(),
                MapStatus::ToggleOff => todo!(),
                _ => {}
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
    pub fn receive_key_event(&mut self, key_name: String, action_type: i32) {
        // If this key does not exist in the dictionary yet, just skip
        if !self.keys.contains_key(&key_name) { return; }   
        let mut map_data = self.keys.get_mut(&key_name).unwrap();    
        match action_type {
            0 => {
                // Set the map status
                map_data.status = MapStatus::Pressed;
            }
            1 => {
                // If this is a button, then set the normal map status
                match map_data.map_type {
                    MapType::Button => {
                        // Set the map status
                        map_data.status = MapStatus::Released;
                    },
                    MapType::Toggle => {
                        // Invert it
                        match map_data.status {
                            MapStatus::ToggleOn => map_data.status = MapStatus::ToggleOff,
                            MapStatus::ToggleOff => map_data.status = MapStatus::ToggleOn,
                            _ => {}
                        }
                    },
                }
                
            }
            _ => {}
        }
    }
    // Binds a key to a specific mapping
    pub fn bind_key(&mut self, key_name: &str, map_name: &str, map_type: MapType) {
        // Check if the binding exists
        if !self.bindings.contains_key(&key_name.to_string()) {
            // The binding does not exist yet, so create a new one
            self.bindings.insert(key_name.to_string(), map_name.to_string());
            self.mappings.insert(map_name.to_string(), MapData::default());
        }
    }    
}

// The get-map events 
impl InputManager {
    // Returns true when the map is pressed
    pub fn map_pressed(&self, name: &str) -> bool {
        // Make sure that mapping actually exists
        if self.mappings.contains_key(&name.to_string()) {
            match self.mappings[&name.to_string()].status {
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
            match self.mappings[&name.to_string()].status {
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
            match self.mappings[&name.to_string()].status {
                MapStatus::Released => true,
                _ => false,
            }
        } else {
            false
        }
    }
    // Returns the toggle state of the map
    pub fn map_toggled(&self, name: &str) -> bool {
        if self.mappings.contains_key(&name.to_string()) {
            match self.mappings[&name.to_string()].status {
                MapStatus::ToggleOn => true,
                MapStatus::ToggleOff => false,
                _ => false,
            }
        } else {
            false
        }
    }
    // Gets the status of a specific map
    pub fn get_map_data(&self, name: String) -> MapData {
        if self.mappings.contains_key(&name) {
            self.mappings[&name]
        } else {
            return MapData::default();
        }
    }
}
