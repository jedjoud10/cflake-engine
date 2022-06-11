use ahash::AHashMap;
use glutin::event::ElementState;


// The virtual keycodes that the window will receive (as a form of events)
pub type Key = glutin::event::VirtualKeyCode;

// The current state of any key
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    Pressed, Released, Held, None
}

impl State {
    // This checks if the state is equal to State::Pressed
    fn pressed(&self) -> bool {
        match self {
            State::Pressed => true,
            _ => false
        }
    }

    // This checks if the state is equal to State::Released
    fn released(&self) -> bool {
        match self {
            State::Released => true,
            _ => false
        }
    }

    // This checks if the State is equal to State::Held
    fn held(&self) -> bool {
        match self {
            State::Held => todo!(),
            _ => false    
        }
    }
}

// An input manager takes keyboard and mouse inputs from the Glutin window and maps them to specific key binds
#[derive(Default)]
pub struct InputManager {
    // Keyboard
    // "forward_key_bind" -> Key::W
    binds: AHashMap<&'static str, Key>,

    // Key::W -> State::Pressed
    keys: AHashMap<Key, State>,

    // Mouse position and mouse scroll values 
    position: vek::Vec2<f32>,
    scroll: f32,
}

impl InputManager {
    /*
    // Called whenever the mouse position changes
    pub fn receive_mouse_position_event(&mut self, delta: vek::Vec2<f32>) {
        self.mouse_pos += delta;
    }
    // Called whenever the mous scroll changes
    pub fn receive_mouse_scroll_event(&mut self, scroll_delta: f32) {
        self.mouse_scroll += scroll_delta;
    }
    */
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

    // This is called whene we receive a new key event from Glutin
    pub fn receive(&mut self, key: Key, state: ElementState) {
        // Update the state for this specific key, and add it if it does not exist
        *self.keys.entry(key).or_insert(State::Pressed) = match state {
            ElementState::Pressed => State::Pressed,
            ElementState::Released => State::Released,
        };
    }

    // Create a new key binding using a name and a unique key
    pub fn bind(&mut self, name: &'static str, key: Key) {
        self.binds.insert(name, key);
    }

    // Get the raw state of a key bind (map)
    pub fn state(&self, name: &'static str) -> Option<State> {
        self.binds.get(name).map(|key| self.keys.get(key).cloned()).flatten()
    }

    // Check if a key was pressed in the current frame
    pub fn pressed(&self, name: &'static str) -> bool {
        self.state(name).map(State::pressed).unwrap_or_default()        
    }

    // Check if a key is being held (a held key is just a key that has been pressed for more than 2 frames)
    pub fn 
}

// The get-map events
impl InputManager {
    // Get the map state of a specific map
    pub fn get(&self, name: &str) -> Option<&MapState> {
        self.maps.get(name).map(|(state, _)| state)
    }
    // Returns true when the map is pressed
    pub fn pressed(&self, name: &str) -> bool {
        self.maps
            .get(name)
            .and_then(|(map_state, _)| {
                if let MapState::Button(ButtonState::Pressed) = map_state {
                    Some(())
                } else {
                    None
                }
            })
            .is_some()
    }
    // Returns true when the map is being held
    pub fn held(&self, name: &str) -> bool {
        self.maps
            .get(name)
            .and_then(|(map_state, _)| {
                if let MapState::Button(ButtonState::Held) = map_state {
                    Some(())
                } else {
                    None
                }
            })
            .is_some()
    }
    // Returns true when the map has been released
    pub fn released(&self, name: &str) -> bool {
        self.maps
            .get(name)
            .and_then(|(map_state, _)| {
                if let MapState::Button(ButtonState::Released) = map_state {
                    Some(())
                } else {
                    None
                }
            })
            .is_some()
    }
    // Check if a map changed
    pub fn changed(&self, name: &str) -> bool {
        self.maps
            .get(name)
            .and_then(|(_, changed)| if *changed { Some(()) } else { None })
            .is_some()
    }
    // Returns the toggle state of the map
    pub fn toggled(&self, name: &str) -> bool {
        self.maps
            .get(name)
            .and_then(|(map_state, _)| {
                if let MapState::Toggle(ToggleState::On) = map_state {
                    Some(())
                } else {
                    None
                }
            })
            .is_some()
    }
}
