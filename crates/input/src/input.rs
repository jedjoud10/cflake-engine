use ahash::{AHashMap, RandomState};
use getset::{CopyGetters, Getters};
use multimap::MultiMap;

use super::{Keys, State};
use crate::{ButtonState, MapState, ToggleState};

// A simple input manager that reads keys from the keyboard and binds them to specific mappings
// Get binding:
// Using the name of the binding, get the scane code for each key and use that scan code to get the map state of that key
#[derive(Getters, CopyGetters)]
pub struct InputManager {
    // "debug_map" -> State: "Pressed"
    #[getset(get = "pub")]
    maps: AHashMap<String, (MapState, bool)>,

    // "W" -> ["forward_map", "launch_map"]
    #[getset(get = "pub")]
    keys: MultiMap<Keys, String, RandomState>,

    // Mouse
    #[getset(get_copy = "pub")]
    mouse_pos: vek::Vec2<f32>,
    #[getset(get_copy = "pub")]
    mouse_pos_delta: vek::Vec2<f32>,
    #[getset(get_copy = "pub")]
    mouse_scroll_delta: f32,
    #[getset(get_copy = "pub")]
    mouse_scroll: f32,
}

impl Default for InputManager {
    fn default() -> Self {
        let multimap = MultiMap::<Keys, String, RandomState>::with_capacity_and_hasher(180, RandomState::new());
        Self {
            maps: Default::default(),
            keys: multimap,
            mouse_pos: Default::default(),
            mouse_pos_delta: Default::default(),
            mouse_scroll: Default::default(),
            mouse_scroll_delta: Default::default(),
        }
    }
}

impl InputManager {
    // Called whenever the mouse position changes
    pub fn receive_mouse_position_event(&mut self, delta: vek::Vec2<f32>) {
        self.mouse_pos += delta;
        self.mouse_pos_delta = delta;
    }
    // Called whenever the mous scroll changes
    pub fn receive_mouse_scroll_event(&mut self, scroll_delta: f32) {
        self.mouse_scroll += scroll_delta;
        self.mouse_scroll_delta = scroll_delta;
    }
    // This should be ran at the end of every frame
    pub fn late_update(&mut self) {
        for (_map_name, (map_state, changed)) in self.maps.iter_mut() {
            // Reset the map state if needed
            *changed = false;
            if let MapState::Button(button_state) = map_state {
                match button_state {
                    ButtonState::Pressed => *button_state = ButtonState::Held,
                    ButtonState::Released => *button_state = ButtonState::Nothing,
                    _ => {}
                }
            }
        }
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
                        MapState::Button(button_state) => match &button_state {
                            ButtonState::Released | ButtonState::Nothing => *button_state = ButtonState::Pressed,
                            _ => {}
                        },
                        MapState::Toggle(toggle_state) => toggle_state.toggle(),
                    }
                }
                State::Released => {
                    // We released the key
                    if let MapState::Button(button_state) = map {
                        *button_state = ButtonState::Released
                    }
                }
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
            .and_then(|(map_state, _)| if let MapState::Button(ButtonState::Pressed) = map_state { Some(()) } else { None })
            .is_some()
    }
    // Returns true when the map is being held
    pub fn map_held(&self, name: &str) -> bool {
        self.maps
            .get(name)
            .and_then(|(map_state, _)| if let MapState::Button(ButtonState::Held) = map_state { Some(()) } else { None })
            .is_some()
    }
    // Returns true when the map has been released
    pub fn map_released(&self, name: &str) -> bool {
        self.maps
            .get(name)
            .and_then(|(map_state, _)| if let MapState::Button(ButtonState::Released) = map_state { Some(()) } else { None })
            .is_some()
    }
    // Check if a map changed
    pub fn map_changed(&self, name: &str) -> bool {
        self.maps.get(name).and_then(|(_, changed)| if *changed { Some(()) } else { None }).is_some()
    }
    // Returns the toggle state of the map
    pub fn map_toggled(&self, name: &str) -> bool {
        self.maps
            .get(name)
            .and_then(|(map_state, _)| if let MapState::Toggle(ToggleState::On) = map_state { Some(()) } else { None })
            .is_some()
    }
}
