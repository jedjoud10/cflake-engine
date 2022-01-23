use super::Keys;
use std::{array::IntoIter, collections::HashMap, iter::FromIterator};

// Status of a key
#[derive(Clone, Copy, Debug)]
pub enum KeyStatus {
    Released,  // The frame the key was released on
    Held(f32), // If the key was held more than a frame
    Pressed,   // The frame the key was pressed on
    Nothing,   // If nothing happens in that specific frame
}

// Toggle status of a key
#[derive(Clone, Copy, Debug)]
pub enum ToggleKeyStatus {
    ToggleOn,
    ToggleOff,
}

impl Default for ToggleKeyStatus {
    fn default() -> Self {
        Self::ToggleOff
    }
}

impl Default for KeyStatus {
    fn default() -> Self {
        Self::Nothing
    }
}

// The type of map
#[derive(Clone, Copy, Debug)]
pub enum MapType {
    Button, // You can press and release it, or even hold it
    Toggle, // Just a toggle
}

// A simple input manager that reads keys from the keyboard and binds them to specific mappings
// Get binding:
// Using the name of the binding, get the scane code for each key and use that scan code to get the map state of that key
pub struct InputManager {
    pub bindings: HashMap<String, i32>,
    pub keys: HashMap<i32, (KeyStatus, ToggleKeyStatus)>,
    scancode_cache: HashMap<Keys, i32>,
    last_mouse_pos: (i32, i32),
    last_mouse_scroll: f32,
}

impl Default for InputManager {
    fn default() -> Self {
        Self {
            bindings: Default::default(),
            keys: Default::default(),
            scancode_cache: Default::default(),
            last_mouse_pos: Default::default(),
            last_mouse_scroll: Default::default(),
        }
    }
}

impl InputManager {
    // Create the key scancode cache
    pub fn create_key_cache(&mut self) {
        let cache: HashMap<Keys, Option<i32>> = HashMap::<Keys, Option<i32>>::from([
            (Keys::Escape, glfw::Key::get_scancode(&glfw::Key::Escape)),
            (Keys::Enter, glfw::Key::get_scancode(&glfw::Key::Enter)),
            (Keys::LeftShift, glfw::Key::get_scancode(&glfw::Key::LeftShift)),
            (Keys::LeftControl, glfw::Key::get_scancode(&glfw::Key::LeftControl)),
            (Keys::RightShift, glfw::Key::get_scancode(&glfw::Key::RightShift)),
            (Keys::RightControl, glfw::Key::get_scancode(&glfw::Key::RightControl)),
            (Keys::Space, glfw::Key::get_scancode(&glfw::Key::Space)),
            (Keys::Minus, glfw::Key::get_scancode(&glfw::Key::Minus)),
            (Keys::Period, glfw::Key::get_scancode(&glfw::Key::Period)),
            (Keys::Comma, glfw::Key::get_scancode(&glfw::Key::Comma)),
            (Keys::Backspace, glfw::Key::get_scancode(&glfw::Key::Backspace)),
            (Keys::A, glfw::Key::get_scancode(&glfw::Key::A)),
            (Keys::B, glfw::Key::get_scancode(&glfw::Key::B)),
            (Keys::C, glfw::Key::get_scancode(&glfw::Key::C)),
            (Keys::D, glfw::Key::get_scancode(&glfw::Key::D)),
            (Keys::E, glfw::Key::get_scancode(&glfw::Key::E)),
            (Keys::F, glfw::Key::get_scancode(&glfw::Key::F)),
            (Keys::G, glfw::Key::get_scancode(&glfw::Key::G)),
            (Keys::H, glfw::Key::get_scancode(&glfw::Key::H)),
            (Keys::I, glfw::Key::get_scancode(&glfw::Key::I)),
            (Keys::J, glfw::Key::get_scancode(&glfw::Key::J)),
            (Keys::K, glfw::Key::get_scancode(&glfw::Key::K)),
            (Keys::L, glfw::Key::get_scancode(&glfw::Key::L)),
            (Keys::M, glfw::Key::get_scancode(&glfw::Key::M)),
            (Keys::N, glfw::Key::get_scancode(&glfw::Key::N)),
            (Keys::O, glfw::Key::get_scancode(&glfw::Key::O)),
            (Keys::P, glfw::Key::get_scancode(&glfw::Key::P)),
            (Keys::Q, glfw::Key::get_scancode(&glfw::Key::Q)),
            (Keys::R, glfw::Key::get_scancode(&glfw::Key::R)),
            (Keys::S, glfw::Key::get_scancode(&glfw::Key::S)),
            (Keys::T, glfw::Key::get_scancode(&glfw::Key::T)),
            (Keys::U, glfw::Key::get_scancode(&glfw::Key::U)),
            (Keys::V, glfw::Key::get_scancode(&glfw::Key::V)),
            (Keys::W, glfw::Key::get_scancode(&glfw::Key::W)),
            (Keys::X, glfw::Key::get_scancode(&glfw::Key::X)),
            (Keys::Y, glfw::Key::get_scancode(&glfw::Key::Y)),
            (Keys::Z, glfw::Key::get_scancode(&glfw::Key::Z)),
            (Keys::F1, glfw::Key::get_scancode(&glfw::Key::F1)),
            (Keys::F2, glfw::Key::get_scancode(&glfw::Key::F2)),
            (Keys::F3, glfw::Key::get_scancode(&glfw::Key::F3)),
            (Keys::F4, glfw::Key::get_scancode(&glfw::Key::F4)),
            (Keys::F5, glfw::Key::get_scancode(&glfw::Key::F5)),
            (Keys::F6, glfw::Key::get_scancode(&glfw::Key::F6)),
            (Keys::F7, glfw::Key::get_scancode(&glfw::Key::F7)),
            (Keys::F8, glfw::Key::get_scancode(&glfw::Key::F8)),
            (Keys::F9, glfw::Key::get_scancode(&glfw::Key::F9)),
            (Keys::F10, glfw::Key::get_scancode(&glfw::Key::F10)),
            (Keys::F11, glfw::Key::get_scancode(&glfw::Key::F11)),
            (Keys::F12, glfw::Key::get_scancode(&glfw::Key::F12)),
            (Keys::NUM1, glfw::Key::get_scancode(&glfw::Key::Num1)),
            (Keys::NUM2, glfw::Key::get_scancode(&glfw::Key::Num2)),
            (Keys::NUM3, glfw::Key::get_scancode(&glfw::Key::Num3)),
            (Keys::NUM4, glfw::Key::get_scancode(&glfw::Key::Num4)),
            (Keys::NUM5, glfw::Key::get_scancode(&glfw::Key::Num5)),
            (Keys::NUM6, glfw::Key::get_scancode(&glfw::Key::Num6)),
            (Keys::NUM7, glfw::Key::get_scancode(&glfw::Key::Num7)),
            (Keys::NUM8, glfw::Key::get_scancode(&glfw::Key::Num8)),
            (Keys::NUM9, glfw::Key::get_scancode(&glfw::Key::Num9)),
            (Keys::NUM0, glfw::Key::get_scancode(&glfw::Key::Num0)),
        ]);
        // Unwrap each value
        let cache = cache.iter().map(|(key, val)| (*key, val.unwrap())).collect::<HashMap<Keys, i32>>();
        self.scancode_cache = cache;
    }
    // Get the key scancode using the cache that we have
    fn get_key_scancode(&self, key: Keys) -> Option<&i32> {
        self.scancode_cache.get(&key)
    }
    // Called when we recieve a new mouse event from the window (Could either be a mouse position one or a scroll one)
    pub fn receive_mouse_event(&mut self, position: Option<(f64, f64)>, scroll: Option<f64>) {
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
            match key.1 .0 {
                KeyStatus::Released => {
                    // Go from "Released" to "Nothing"
                    key.1 .0 = KeyStatus::Nothing;
                }
                KeyStatus::Held(old_time) => {
                    // Add delta time to the held seconds counter
                    key.1 .0 = KeyStatus::Held(old_time + delta_time);
                }
                KeyStatus::Pressed => {
                    // Go from "Pressed" to "Held"
                    key.1 .0 = KeyStatus::Held(0.0);
                }
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
    // When we receive a key event from glfw (Always at the start of the frame)
    pub fn receive_key_event(&mut self, key_scancode: i32, action_type: i32) {
        // If this key does not exist in the dictionary yet, add it
        let mut key_data = self.keys.entry(key_scancode).or_insert((KeyStatus::default(), ToggleKeyStatus::default()));
        match action_type {
            0 => {
                // Set the key status
                key_data.0 = KeyStatus::Pressed;
                // Update the toggle on key press
                match key_data.1 {
                    ToggleKeyStatus::ToggleOn => key_data.1 = ToggleKeyStatus::ToggleOff,
                    ToggleKeyStatus::ToggleOff => key_data.1 = ToggleKeyStatus::ToggleOn,
                }
            }
            1 => {
                // Set the key status
                key_data.0 = KeyStatus::Released;
            }
            _ => {}
        }
    }
    // Binds a key to a specific mapping
    pub fn bind_key(&mut self, key: Keys, map_name: &str, _map_type: MapType) {
        // Check if the binding exists
        let key_scancode = *self.get_key_scancode(key).unwrap();
        if !self.bindings.contains_key(map_name) {
            // The binding does not exist yet, so create a new one
            self.bindings.insert(map_name.to_string(), key_scancode);
        }
    }
}

// The get-map events
impl InputManager {
    // Returns true when the map is pressed
    pub fn map_pressed(&self, name: &str) -> bool {
        // Make sure that mapping actually exists
        if self.bindings.contains_key(&name.to_string()) {
            let key_scancode = self.bindings.get(&name.to_string()).unwrap();
            if self.keys.contains_key(key_scancode) {
                match self.keys.get(key_scancode).unwrap().0 {
                    KeyStatus::Pressed => true,
                    _ => false,
                }
            } else {
                false
            }
        } else {
            false
        }
    }
    // Returns true when the map is pressed, ignores the update check
    pub fn map_pressed_uncheck(&self, name: &str) -> bool {
        // Make sure that mapping actually exists
        if self.bindings.contains_key(&name.to_string()) {
            let key_scancode = self.bindings.get(&name.to_string()).unwrap();
            if self.keys.contains_key(key_scancode) {
                match self.keys.get(key_scancode).unwrap().0 {
                    KeyStatus::Pressed => true,
                    _ => false,
                }
            } else {
                false
            }
        } else {
            false
        }
    }
    // Returns true when the map is being held
    pub fn map_held(&self, name: &str) -> (bool, f32) {
        // Make sure that mapping actually exists
        if self.bindings.contains_key(&name.to_string()) {
            let key_scancode = self.bindings.get(&name.to_string()).unwrap();
            if self.keys.contains_key(key_scancode) {
                match self.keys.get(key_scancode).unwrap().0 {
                    KeyStatus::Held(held_seconds) => (true, held_seconds),
                    _ => (false, 0.0),
                }
            } else {
                (false, 0.0)
            }
        } else {
            (false, 0.0)
        }
    }
    // Returns true when the map has been released
    pub fn map_released(&self, name: &str) -> bool {
        if self.bindings.contains_key(&name.to_string()) {
            let key_scancode = self.bindings.get(&name.to_string()).unwrap();
            if self.keys.contains_key(key_scancode) {
                match self.keys.get(key_scancode).unwrap().0 {
                    KeyStatus::Released => true,
                    _ => false,
                }
            } else {
                false
            }
        } else {
            false
        }
    }
    // Check if the toggle state of the map has changed
    pub fn map_toggle_changed(&self, name: &str) -> bool {
        if self.bindings.contains_key(&name.to_string()) {
            let key_scancode = self.bindings.get(&name.to_string()).unwrap();
            if self.keys.contains_key(key_scancode) {
                match self.keys.get(key_scancode).unwrap().0 {
                    KeyStatus::Pressed => true,
                    _ => false,
                }
            } else {
                false
            }
        } else {
            false
        }
    }
    // Returns the toggle state of the map
    pub fn map_toggled(&self, name: &str) -> bool {
        if self.bindings.contains_key(&name.to_string()) {
            let key_scancode = self.bindings.get(&name.to_string()).unwrap();
            if self.keys.contains_key(key_scancode) {
                match self.keys.get(key_scancode).unwrap().1 {
                    ToggleKeyStatus::ToggleOn => true,
                    ToggleKeyStatus::ToggleOff => false,
                }
            } else {
                false
            }
        } else {
            false
        }
    }
}
