use crate::{Key, KeyState};
use ahash::AHashMap;
use world::Resource;

// This keyboard struct will be responsible for all key events and state handling for the keyboard
#[derive(Resource)]
pub struct Keyboard {
    // "forward_key_bind" -> Key::W
    pub(crate) binds: AHashMap<&'static str, Key>,

    // Key::W -> State::Pressed
    pub(crate) keys: AHashMap<Key, KeyState>,
}

impl Keyboard {
    // Create a new key binding using a name and a unique key
    pub fn bind(&mut self, name: &'static str, key: Key) {
        self.binds.insert(name, key);
    }

    // Get the raw state of a unique key
    pub fn key(&self, key: Key) -> KeyState {
        self.keys.get(&key).cloned().unwrap_or(KeyState::None)
    }

    // Get the raw state of a key bind (map)
    pub fn state(&self, name: &'static str) -> Option<&KeyState> {
        self.binds.get(name).and_then(|key| self.keys.get(key))
    }

    // Check if a keybind was pressed in the current frame
    pub fn pressed(&self, name: &'static str) -> bool {
        self.state(name).map(KeyState::pressed).unwrap_or_default()
    }

    // Check if a keybind is being held (a held key is just a key that has been pressed for more than 2 frames)
    pub fn held(&self, name: &'static str) -> bool {
        self.state(name).map(KeyState::held).unwrap_or_default()
    }

    // Check if a keybind was released in the current frame
    pub fn released(&self, name: &'static str) -> bool {
        self.state(name).map(KeyState::released).unwrap_or_default()
    }
}
