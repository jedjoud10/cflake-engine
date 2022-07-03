use glutin::event::ElementState;

// The virtual keycodes that the window will receive (as a form of events)
pub type Key = glutin::event::VirtualKeyCode;

// The current state of any key
#[derive(Clone, Copy, PartialEq, Eq)]
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
