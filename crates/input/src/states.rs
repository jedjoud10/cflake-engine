
// State for a button map
#[derive(Clone, Copy)]
pub enum ButtonState {
    Pressed, Released, Held, Nothing
}

impl Default for ButtonState {
    fn default() -> Self {
        Self::Nothing
    }
}

// State for a toggle map
#[derive(Clone, Copy)]
pub enum ToggleState {
    On, Off
}

impl ToggleState {
    pub fn toggle(&mut self) {
        match self {
            ToggleState::On => *self = Self::Off,
            ToggleState::Off => *self = Self::On,
        }
    }
}

impl Default for ToggleState {
    fn default() -> Self {
        Self::Off
    }
}

// General map state
#[derive(Clone, Copy)]
pub enum MapState {
    Button(ButtonState),
    Toggle(ToggleState),
}

impl MapState {
    // Default toggle
}