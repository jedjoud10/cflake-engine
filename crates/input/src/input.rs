use ahash::AHashMap;
use glutin::event::ElementState;

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

// An axis can be mapped to a specific binding to be able to fetch it using a user defined name
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Axis {
    MousePositionX,
    MousePositionY,
    MouseScroll,
    MousePositionDeltaX,
    MousePositionDeltaY,
    MouseScrollDelta,
}

// This keyboard struct will be responsible for all key events and state handling for the keyboard
pub struct Input {
    // "forward_key_bind" -> Key::W
    pub(crate) key_bindings: AHashMap<&'static str, Key>,

    // Key::W -> State::Pressed
    pub(crate) keys: AHashMap<Key, KeyState>,

    // "camera rotation" -> Axis:MousePositionX,
    pub(crate) axis_bindings: AHashMap<&'static str, Axis>,

    // Axis::MousePositionX -> 561.56
    pub(crate) axii: AHashMap<Axis, f32>,
}

// Trait implemented for structs that allow us to fetch the key state from the main input handler
pub trait InputButtonId {
    fn get(self, input: &Input) -> KeyState;
}

impl InputButtonId for Key {
    fn get(self, input: &Input) -> KeyState {
        input.keys.get(&self).cloned().unwrap_or(KeyState::None)
    }
}

impl InputButtonId for &'static str {
    fn get(self, input: &Input) -> KeyState {
        input
            .key_bindings
            .get(self)
            .map(|key| Key::get(*key, input))
            .unwrap_or(KeyState::None)
    }
}

// Trait implemented for structs that allow us to fetch the axis state from the main input handler
pub trait InputAxisId {
    fn get(self, input: &Input) -> f32;
}

impl InputAxisId for Axis {
    fn get(self, input: &Input) -> f32 {
        input.axii.get(&self).cloned().unwrap_or_default()
    }
}

impl InputAxisId for &'static str {
    fn get(self, input: &Input) -> f32 {
        input
            .axis_bindings
            .get(self)
            .map(|axis| Axis::get(*axis, input))
            .unwrap_or_default()
    }
}

impl Input {
    // Create a new button binding using a name and a unique key
    pub fn bind_key(&mut self, name: &'static str, key: Key) {
        self.key_bindings.insert(name, key);
    }

    // Create a new axis binding using a name and a unique axis
    pub fn bind_axis(&mut self, name: &'static str, axis: Axis) {
        self.axis_bindings.insert(name, axis);
    }

    // Get the state of a button mapping or a key mapping
    pub fn key<B: InputButtonId>(&self, button: B) -> KeyState {
        B::get(button, self)
    }

    // Get the state of a unique axis or an axis mapping
    pub fn axis<A: InputAxisId>(&self, axis: A) -> f32 {
        A::get(axis, self)
    }
}
