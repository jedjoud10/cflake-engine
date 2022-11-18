mod system;
pub use system::*;

use ahash::AHashMap;
use serde::*;
use std::{
    borrow::Cow,
    fs::File,
    io::{BufReader, Read, Write},
    path::Path,
};
use winit::event::ElementState;

// The virtual keycodes that the window will receive (as a form of events)
pub type Key = winit::event::VirtualKeyCode;

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
        matches!(self, KeyState::Pressed)
    }

    // This checks if the state is equal to State::Released
    pub fn released(&self) -> bool {
        matches!(self, KeyState::Released)
    }

    // This checks if the State is equal to State::Held
    pub fn held(&self) -> bool {
        matches!(self, KeyState::Held)
    }
}

// An axis can be mapped to a specific binding to be able to fetch it using a user defined name
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    // Key and axis bindings
    pub(crate) bindings: InputUserBindings,

    // Key::W -> State::Pressed
    pub(crate) keys: AHashMap<Key, KeyState>,

    // Axis::MousePositionX -> 561.56
    pub(crate) axii: AHashMap<Axis, f32>,
}

// User input bindings are basically
#[derive(Default, Clone, Serialize, Deserialize)]
// TODO: Sort by string name
pub struct InputUserBindings {
    // "forward_key_bind" -> Key::W
    pub(crate) key_bindings: AHashMap<Cow<'static, str>, Key>,

    // "camera rotation" -> Axis:MousePositionX,
    pub(crate) axis_bindings: AHashMap<Cow<'static, str>, Axis>,
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
            .bindings
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
            .bindings
            .axis_bindings
            .get(self)
            .map(|axis| Axis::get(*axis, input))
            .unwrap_or_default()
    }
}

impl Input {
    // Load the bindings from the user binding struct
    // If there are conflicting bindings, they will get overwritten
    pub fn read_bindings_from_user_bindings(&mut self, user: InputUserBindings) {
        self.bindings.axis_bindings.extend(user.axis_bindings);
        self.bindings.key_bindings.extend(user.key_bindings);
    }

    // Load the bindings from a file
    // If there are conflicting bindings, they will get overwritten
    pub fn read_bindings_from_file<P: AsRef<Path>>(&mut self, path: P) -> Option<()> {
        let mut options = File::options();
        options.read(true);
        let mut string = String::new();
        let mut reader = BufReader::new(options.open(path).ok()?);
        reader.read_to_string(&mut string).unwrap();
        let cow = Cow::from(string);

        let bindings = serde_json::from_str(&cow).ok()?;
        self.read_bindings_from_user_bindings(bindings);
        Some(())
    }

    // Convert the bindings to a user binding struct
    pub fn as_user_binding(&self) -> InputUserBindings {
        self.bindings.clone()
    }

    // Write the bindings to a file
    // If the file does not exist, create it
    pub fn write_bindings_to_file<P: AsRef<Path>>(&self, path: P) -> Option<()> {
        let mut options = File::options();
        options.read(true);
        options.write(true);
        options.truncate(true);
        options.create_new(true);
        let mut file = options.open(path).ok()?;
        let data = self.as_user_binding();
        let json = serde_json::to_string_pretty(&data).ok()?;
        file.write_all(json.as_bytes()).ok()?;
        Some(())
    }

    // Create a new button binding using a name and a unique key
    pub fn bind_key(&mut self, name: &'static str, key: Key) {
        self.bindings.key_bindings.insert(Cow::Borrowed(name), key);
    }

    // Create a new axis binding using a name and a unique axis
    pub fn bind_axis(&mut self, name: &'static str, axis: Axis) {
        self.bindings
            .axis_bindings
            .insert(Cow::Borrowed(name), axis);
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
