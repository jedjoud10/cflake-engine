#![warn(missing_docs)]

//! TODO: Docs

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
use std::collections::BTreeMap;

/// Main input resource responsible for keyboard / mouse / gamepad input events.
/// This resource will automatically be added into the world at startup.
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
}

/// User input bindings that can be serialized / deserialized.
#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'static"))]
pub struct InputUserBindings {
    // "forward_key_bind" -> Key::W
    #[serde(serialize_with = "order")]
    pub(crate) key_bindings: AHashMap<&'static str, Button>,

    // "camera rotation" -> Axis:MousePositionX,
    #[serde(serialize_with = "order")]
    pub(crate) axis_bindings: AHashMap<&'static str, Axis>,
}

fn order<S, V: Serialize>(value: &AHashMap<&'static str, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}

impl Input {
    /// Load the bindings from the user binding struct.
    /// If there are conflicting bindings, they will get overwritten.
    pub fn read_bindings_from_user_bindings(&mut self, user: InputUserBindings) {
        self.bindings.axis_bindings.extend(user.axis_bindings);
        self.bindings.key_bindings.extend(user.key_bindings);
    }

    /// Convert the bindings to a user binding struct.
    pub fn as_user_binding(&self) -> InputUserBindings {
        self.bindings.clone()
    }

    /// Create a new button binding using a name and a unique key.
    pub fn bind_button(&mut self, name: &'static str, key: impl Into<Button>) {
        let key = key.into();
        log::debug!("Binding button/key {key:?} to '{name}'");
        self.bindings.key_bindings.insert(name, key);
    }

    /// Create a new axis binding using a name and a unique axis.
    pub fn bind_axis(&mut self, name: &'static str, axis: impl Into<Axis>) {
        let axis = axis.into();
        log::debug!("Binding axis {axis:?} to '{name}'");
        self.bindings
            .axis_bindings
            .insert(name, axis);
    }

    /// Get the state of a button mapping or a key mapping.
    pub fn get_button<B: InputButtonId>(&self, button: B) -> ButtonState {
        B::get(button, self)
    }

    /// Get the state of a unique axis or an axis mapping.
    pub fn get_axis<A: InputAxisId>(&self, axis: A) -> f32 {
        A::get(axis, self)
    }
}
