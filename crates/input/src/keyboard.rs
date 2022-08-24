use glutin::event::ElementState;
use ahash::AHashMap;

// The virtual keycodes that the window will receive (as a form of events)
pub type Key = glutin::event::VirtualKeyCode;

// The current state of any key
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    Pressed,
    Released,
    Held,
    None,
}

impl From<ElementState> for KeyState {
    fn from(state: ElementState) -> Self {
        match state {
            ElementState::Pressed => Self::Pressed,
            ElementState::Released => Self::Released,
        }
    }
}

impl KeyState {
    // This checks if the state is equal to State::Pressed
    pub fn pressed(&self) -> bool {
        match self {
            KeyState::Pressed => true,
            _ => false,
        }
    }

    // This checks if the state is equal to State::Released
    pub fn released(&self) -> bool {
        match self {
            KeyState::Released => true,
            _ => false,
        }
    }

    // This checks if the State is equal to State::Held
    pub fn held(&self) -> bool {
        match self {
            KeyState::Held => true,
            _ => false,
        }
    }
}


// This keyboard struct will be responsible for all key events and state handling for the keyboard
pub struct Keyboard {
    // "forward_key_bind" -> Key::W
    pub(crate) binds: AHashMap<&'static str, Key>,

    // Key::W -> State::Pressed
    pub(crate) keys: AHashMap<Key, KeyState>,
}

// This is a value that can fetch it's own state
pub trait KeyStateFetcher: Clone {
    fn fetch(self, keyboard: &Keyboard) -> Option<&KeyState>;
}

// Fetch the state of a single key
impl KeyStateFetcher for Key {
    fn fetch(self, keyboard: &Keyboard) -> Option<&KeyState> {
        keyboard.keys.get(&self)
    }
}

// Fetch the state of a single mapping
impl KeyStateFetcher for &'static str {
    fn fetch(self, keyboard: &Keyboard) -> Option<&KeyState> {
        keyboard.binds.get(self).and_then(|key| keyboard.keys.get(key))
    }
}

impl Keyboard {
    // Create a new key binding using a name and a unique key
    pub fn bind(&mut self, name: &'static str, key: Key) {
        self.binds.insert(name, key);
    }

    // Get the raw state of a unique key
    pub fn state(&self, id: impl KeyStateFetcher) -> KeyState {
        id.fetch(self).cloned().unwrap_or(KeyState::None)
    }

    // Check if a keybind was pressed in the current frame
    pub fn pressed(&self, id: impl KeyStateFetcher) -> bool {
        id.fetch(self).map(KeyState::pressed).unwrap_or_default()
    }

    // Check if a keybind is being held (a held key is just a key that has been pressed for more than 2 frames)
    pub fn held(&self, id: impl KeyStateFetcher) -> bool {
        id.fetch(self).map(KeyState::held).unwrap_or_default()
    }

    // Check if a keybind was released in the current frame
    pub fn released(&self, id: impl KeyStateFetcher) -> bool {
        id.fetch(self).map(KeyState::released).unwrap_or_default()
    }
}
