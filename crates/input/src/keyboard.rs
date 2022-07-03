use crate::{Key, State};
use ahash::AHashMap;

// This keyboard struct will be responsible for all key events and state handling for the keyboard
pub struct Keyboard {
    // "forward_key_bind" -> Key::W
    pub(crate) binds: AHashMap<&'static str, Key>,

    // Key::W -> State::Pressed
    pub(crate) keys: AHashMap<Key, State>,
}

impl Keyboard {
    // Create a new key binding using a name and a unique key
    pub fn bind(&mut self, name: &'static str, key: Key) {
        self.binds.insert(name, key);
    }

    // Get the raw state of a key bind (map)
    pub fn state(&self, name: &'static str) -> Option<&State> {
        self.binds.get(name).and_then(|key| self.keys.get(key))
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
