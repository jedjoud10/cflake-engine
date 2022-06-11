use ahash::AHashMap;
use glutin::event::ElementState;
use world::resources::Resource;


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
pub struct Input {
    // Keyboard
    // "forward_key_bind" -> Key::W
    binds: AHashMap<&'static str, Key>,

    // Key::W -> State::Pressed
    keys: AHashMap<Key, State>,

    // Mouse position and mouse scroll values 
    position: vek::Vec2<f32>,
    scroll: f32,
}

impl Resource for Input {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn end_frame(&mut self) {
        // Convert all the State::Pressed keys to State::Held and all the State::Released to State::None
        for (key, state) in self.keys.iter_mut() {
            *state = match state {
                State::Pressed => State::Held,
                State::Released => State::None,
                State::Held => State::Held,
                State::None => State::None,
            };
        }
    }
}

impl Input {
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
    pub fn state(&self, name: &'static str) -> Option<&State> {
        self.binds.get(name).map(|key| self.keys.get(key)).flatten()
    }

    // Check if a keybind was pressed in the current frame
    pub fn pressed(&self, name: &'static str) -> bool {
        self.state(name).map(State::pressed).unwrap_or_default()        
    }

    // Check if a keybind is being held (a held key is just a key that has been pressed for more than 2 frames)
    pub fn held(&self, name: &'static str) -> bool {
        self.state(name).map(State::held).unwrap_or_default()   
    }

    // Check if a keybind was released in the current frame
    pub fn released(&self, name: &'static str) -> bool {
        self.state(name).map(State::released).unwrap_or_default()   
    }
}