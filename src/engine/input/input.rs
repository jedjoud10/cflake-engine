use std::{
    collections::HashMap,
    fmt::{self},
};
use super::Keys;

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
// Get binding:
// Using the name of the binding, get the map state of the key
// Update bindings:
// Loop through each binding, get it's scancode, use that scan code to get the MapStatus from 'keys'
#[derive(Default)]
pub struct InputManager {
    pub bindings: HashMap<i32, String>,
    pub keys: HashMap<i32, MapData>,
    pub mappings: HashMap<String, MapData>,
    scancode_cache: HashMap<Keys, i32>,
    last_mouse_pos: (i32, i32),
    last_mouse_scroll: f32,
}

impl InputManager {
    // Get the key scancode for a specific key
    pub fn get_key_scancode(key: Keys) -> Option<i32> {
        match key {
            Keys::Escape => glfw::Key::get_scancode(&glfw::Key::Escape),
            Keys::Enter => glfw::Key::get_scancode(&glfw::Key::Enter),
            Keys::LeftShift => glfw::Key::get_scancode(&glfw::Key::LeftShift),
            Keys::LeftControl => glfw::Key::get_scancode(&glfw::Key::LeftControl),
            Keys::RightShift => glfw::Key::get_scancode(&glfw::Key::RightShift),
            Keys::RightControl => glfw::Key::get_scancode(&glfw::Key::RightControl),
            Keys::Space => glfw::Key::get_scancode(&glfw::Key::Space),
            Keys::A => glfw::Key::get_scancode(&glfw::Key::A),
            Keys::B => glfw::Key::get_scancode(&glfw::Key::B),
            Keys::C => glfw::Key::get_scancode(&glfw::Key::C),
            Keys::D => glfw::Key::get_scancode(&glfw::Key::D),
            Keys::E => glfw::Key::get_scancode(&glfw::Key::E),
            Keys::F => glfw::Key::get_scancode(&glfw::Key::F),
            Keys::G => glfw::Key::get_scancode(&glfw::Key::G),
            Keys::H => glfw::Key::get_scancode(&glfw::Key::H),
            Keys::I => glfw::Key::get_scancode(&glfw::Key::I),
            Keys::J => glfw::Key::get_scancode(&glfw::Key::J),
            Keys::K => glfw::Key::get_scancode(&glfw::Key::K),
            Keys::L => glfw::Key::get_scancode(&glfw::Key::L),
            Keys::M => glfw::Key::get_scancode(&glfw::Key::M),
            Keys::N => glfw::Key::get_scancode(&glfw::Key::N),
            Keys::O => glfw::Key::get_scancode(&glfw::Key::O),
            Keys::P => glfw::Key::get_scancode(&glfw::Key::P),
            Keys::Q => glfw::Key::get_scancode(&glfw::Key::Q),
            Keys::R => glfw::Key::get_scancode(&glfw::Key::R),
            Keys::S => glfw::Key::get_scancode(&glfw::Key::S),
            Keys::T => glfw::Key::get_scancode(&glfw::Key::T),
            Keys::U => glfw::Key::get_scancode(&glfw::Key::U),
            Keys::V => glfw::Key::get_scancode(&glfw::Key::V),
            Keys::W => glfw::Key::get_scancode(&glfw::Key::W),
            Keys::X => glfw::Key::get_scancode(&glfw::Key::X),
            Keys::Y => glfw::Key::get_scancode(&glfw::Key::Y),
            Keys::Z => glfw::Key::get_scancode(&glfw::Key::Z),
            Keys::F1 => glfw::Key::get_scancode(&glfw::Key::F1),
            Keys::F2 => glfw::Key::get_scancode(&glfw::Key::F2),
            Keys::F3 => glfw::Key::get_scancode(&glfw::Key::F3),
            Keys::F4 => glfw::Key::get_scancode(&glfw::Key::F4),
            Keys::F5 => glfw::Key::get_scancode(&glfw::Key::F5),
            Keys::F6 => glfw::Key::get_scancode(&glfw::Key::F6),
            Keys::F7 => glfw::Key::get_scancode(&glfw::Key::F7),
            Keys::F8 => glfw::Key::get_scancode(&glfw::Key::F8),
            Keys::F9 => glfw::Key::get_scancode(&glfw::Key::F9),
            Keys::F10 => glfw::Key::get_scancode(&glfw::Key::F10),
            Keys::F11 => glfw::Key::get_scancode(&glfw::Key::F11),
            Keys::F12 => glfw::Key::get_scancode(&glfw::Key::F12),
        }
    }
    // Setup the default input bindings
    pub fn setup_default_bindings(&mut self) {
        self.bind_key(Keys::Escape, "quit", MapType::Button);
        self.bind_key(Keys::F1, "fullscreen", MapType::Button);
        self.bind_key(Keys::F2, "capture_fps", MapType::Button);
        self.bind_key(Keys::F3, "change_debug_view", MapType::Button);
        self.bind_key(Keys::F, "toggle_wireframe", MapType::Button);
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
    pub fn receive_key_event(&mut self, key_scancode: i32, action_type: i32) {
        // If this key does not exist in the dictionary yet, add it       
        let mut map_data =  self.keys.entry(key_scancode).or_insert(MapData::default());
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
    pub fn bind_key(&mut self, key: Keys, map_name: &str, map_type: MapType) {
        // Check if the binding exists
        let key_scancode = Self::get_key_scancode(key).unwrap();
        if !self.bindings.contains_key(&key_scancode) {
            // The binding does not exist yet, so create a new one
            self.bindings.insert(key_scancode, map_name.to_string());
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
