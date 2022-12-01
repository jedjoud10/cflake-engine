mod axis;
mod button;
mod ids;
mod system;
pub use axis::*;
pub use button::*;
pub use ids::*;
pub use system::*;

use ahash::AHashMap;
use serde::*;
use std::{
    borrow::Cow,
    fs::File,
    io::{BufReader, Read, Write},
    path::Path,
};

// This keyboard struct will be responsible for all key events and state handling for the keyboard
pub struct Input {
    // Key and axis bindings
    pub(crate) bindings: InputUserBindings,

    // Key::W -> State::Pressed
    pub(crate) keys: AHashMap<Button, ButtonState>,

    // Axis::MousePositionX -> 561.56
    pub(crate) axii: AHashMap<Axis, f32>,

    // Used only for gamepad support
    pub(crate) gilrs: gilrs::Gilrs,
    pub(crate) gamepad: Option<gilrs::GamepadId>,

    // Last inutted sequence of letters in the window
    #[cfg(feature = "sentence-recording")]
    pub(crate) sentence: Option<String>,
    #[cfg(feature = "sentence-recording")]
    pub(crate) sentence_nl_action: NewLineAction,
}

// User input bindings are basically
#[derive(Default, Clone, Serialize, Deserialize)]
// TODO: Sort by string name
pub struct InputUserBindings {
    // "forward_key_bind" -> Key::W
    pub(crate) key_bindings: AHashMap<Cow<'static, str>, Button>,

    // "camera rotation" -> Axis:MousePositionX,
    pub(crate) axis_bindings: AHashMap<Cow<'static, str>, Axis>,
}

impl Input {
    // Load the bindings from the user binding struct
    // If there are conflicting bindings, they will get overwritten
    pub fn read_bindings_from_user_bindings(
        &mut self,
        user: InputUserBindings,
    ) {
        self.bindings.axis_bindings.extend(user.axis_bindings);
        self.bindings.key_bindings.extend(user.key_bindings);
    }

    // Convert the bindings to a user binding struct
    pub fn as_user_binding(&self) -> InputUserBindings {
        self.bindings.clone()
    }

    // Create a new button binding using a name and a unique key
    pub fn bind_button(&mut self, name: &'static str, key: Button) {
        self.bindings.key_bindings.insert(Cow::Borrowed(name), key);
    }

    // Create a new axis binding using a name and a unique axis
    pub fn bind_axis(&mut self, name: &'static str, axis: Axis) {
        self.bindings
            .axis_bindings
            .insert(Cow::Borrowed(name), axis);
    }

    // Get the state of a button mapping or a key mapping
    pub fn get_button<B: InputButtonId>(
        &self,
        button: B,
    ) -> ButtonState {
        B::get(button, self)
    }

    // Get the state of a unique axis or an axis mapping
    pub fn get_axis<A: InputAxisId>(&self, axis: A) -> f32 {
        A::get(axis, self)
    }
}

#[cfg(feature = "sentence-recording")]
impl Input {
    // Enable sentence recording
    pub fn enable_sentence_recording(&mut self) {
        self.sentence = Some(String::new());
    }

    // Disable sentence recording
    pub fn disable_sentence_recording(&mut self) {
        self.sentence.take();
    }

    // Get the current sentence
    pub fn get_sentence(&self) -> Option<&str> {
        self.sentence.as_ref().map(|s| s.as_str())
    }

    // Action that should be done whenever we press enter
    pub fn set_sentence_new_line_action(
        &mut self,
        action: NewLineAction,
    ) {
        self.sentence_nl_action = action;
    }
}

// Action that should occur whenever you press enter
#[cfg(feature = "sentence-recording")]
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum NewLineAction {
    #[default]
    // Clear the string to an empty string
    Clear,

    // Continue the string without resetting it
    NewLine,
}
