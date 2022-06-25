// The virtual keycodes that the window will receive (as a form of events)
pub type Key = glutin::event::VirtualKeyCode;

// The current state of any key
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    Pressed,
    Released,
    Held,
    None,
}

impl State {
    // This checks if the state is equal to State::Pressed
    pub fn pressed(&self) -> bool {
        match self {
            State::Pressed => true,
            _ => false,
        }
    }

    // This checks if the state is equal to State::Released
    pub fn released(&self) -> bool {
        match self {
            State::Released => true,
            _ => false,
        }
    }

    // This checks if the State is equal to State::Held
    pub fn held(&self) -> bool {
        match self {
            State::Held => todo!(),
            _ => false,
        }
    }
}
