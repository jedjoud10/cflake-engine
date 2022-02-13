use multimap::MultiMap;

use super::{Keys, State};
use crate::{ButtonState, MapState, ToggleState};
use std::collections::HashMap;

// A simple input manager that reads keys from the keyboard and binds them to specific mappings
// Get binding:
// Using the name of the binding, get the scane code for each key and use that scan code to get the map state of that key
pub struct InputManager {
    // "debug_map" -> State: "Pressed"
    maps: HashMap<String, (MapState, bool)>,

    // "W" -> ["forward_map", "launch_map"]
    keys: MultiMap<Keys, String>,

    // Keys
    // Mouse
    last_mouse_pos: veclib::Vector2<f64>,
    last_mouse_scroll: f64,
}

impl Default for InputManager {
    fn default() -> Self {
        Self {
            maps: Default::default(),
            keys: MultiMap::with_capacity(180),
            last_mouse_pos: Default::default(),
            last_mouse_scroll: Default::default(),
        }
    }
}

impl InputManager {
    // Called whenever the mouse position changes
    pub fn receive_mouse_position_event(&mut self, position: veclib::Vector2<f64>) {
        self.last_mouse_pos = position;
    }
    // Called whenever the mous scroll changes
    pub fn receive_mouse_scroll_event(&mut self, scroll_delta: f64) {
        self.last_mouse_scroll += scroll_delta;
    }
    // This should be ran at the start of every frame, before we poll any glfw events
    pub fn late_update(&mut self, _delta_time: f32) {
        for (_map_name, (map_state, changed)) in self.maps.iter_mut() {
            // Reset the map state if needed
            *changed = false;
            match map_state {
                MapState::Button(button_state) => match button_state {
                    ButtonState::Pressed => *button_state = ButtonState::Held,
                    ButtonState::Released => *button_state = ButtonState::Nothing,
                    _ => {}
                },
                _ => {}
            }
        }
    }
    // Get the accumulated mouse position
    pub fn get_mouse_position(&self) -> &veclib::Vector2<f64> {
        &self.last_mouse_pos
    }
    // Get the accumulated mouse scroll
    pub fn get_mouse_scroll(&self) -> &f64 {
        &self.last_mouse_scroll
    }
    // When we receive a key event from winit
    pub fn receive_key_event(&mut self, key: Keys, state: State) -> Option<()> {
        let maps_to_update = self.keys.get_vec(&key)?;
        // Update each map now
        for map_name in maps_to_update {
            let (map, changed) = self.maps.get_mut(map_name)?;
            *changed = true;
            match state {
                State::Pressed => {
                    // We pressed the key
                    match map {
                        MapState::Button(button_state) => *button_state = ButtonState::Pressed,
                        MapState::Toggle(toggle_state) => toggle_state.toggle(),
                    }
                }
                State::Released => {
                    // We released the key
                    match map {
                        MapState::Button(button_state) => *button_state = ButtonState::Released,
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        Some(())
    }
    // Binds a key to a specific mapping, making it a button
    pub fn bind_key(&mut self, key: Keys, map_name: &str) {
        // Check if the binding exists
        if !self.maps.contains_key(map_name) {
            // The binding does not exist yet, so create a new one
            let map_name = map_name.to_string();
            self.maps.insert(map_name.clone(), (MapState::Button(ButtonState::default()), false));
            self.keys.insert(key, map_name);
        }
    }
    pub fn bind_key_toggle(&mut self, key: Keys, map_name: &str) {
        // Check if the binding exists
        if !self.maps.contains_key(map_name) {
            // The binding does not exist yet, so create a new one
            let map_name = map_name.to_string();
            self.maps.insert(map_name.clone(), (MapState::Toggle(ToggleState::default()), false));
            self.keys.insert(key, map_name);
        }
    }
}

// The get-map events
impl InputManager {
    // Returns true when the map is pressed
    pub fn map_pressed(&self, name: &str) -> bool {
        self.maps
            .get(name)
            .and_then(|(map_state, _)| {
                if let MapState::Button(button_state) = map_state {
                    if let ButtonState::Pressed = button_state {
                        Some(())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .is_some()
    }
    // Returns true when the map is being held
    pub fn map_held(&self, name: &str) -> bool {
        self.maps
            .get(name)
            .and_then(|(map_state, _)| {
                if let MapState::Button(button_state) = map_state {
                    if let ButtonState::Held = button_state {
                        Some(())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .is_some()
    }
    // Returns true when the map has been released
    pub fn map_released(&self, name: &str) -> bool {
        self.maps
            .get(name)
            .and_then(|(map_state, _)| {
                if let MapState::Button(button_state) = map_state {
                    if let ButtonState::Released = button_state {
                        Some(())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .is_some()
    }
    // Check if a map changed
    pub fn map_changed(&self, name: &str) -> bool {
        self.maps.get(name).and_then(|(_, changed)| changed.then_some(())).is_some()
    }
    // Returns the toggle state of the map
    pub fn map_toggled(&self, name: &str) -> bool {
        self.maps
            .get(name)
            .and_then(|(map_state, _)| {
                if let MapState::Toggle(toggle_state) = map_state {
                    if let ToggleState::On = toggle_state {
                        Some(())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .is_some()
    }
}
