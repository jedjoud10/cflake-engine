use serde::*;
use winit::event::ElementState;

// The current state of any key
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Pressed,
    Released,
    Held,
    None,
}

impl From<ElementState> for ButtonState {
    fn from(state: ElementState) -> Self {
        match state {
            ElementState::Pressed => Self::Pressed,
            ElementState::Released => Self::Released,
        }
    }
}

impl ButtonState {
    // This checks if the state is equal to State::Pressed
    pub fn pressed(&self) -> bool {
        matches!(self, ButtonState::Pressed)
    }

    // This checks if the state is equal to State::Released
    pub fn released(&self) -> bool {
        matches!(self, ButtonState::Released)
    }

    // This checks if the State is equal to State::Held
    pub fn held(&self) -> bool {
        matches!(self, ButtonState::Held)
    }
}

// Convert a winit VirtualKeyCode to an input button
pub fn from_winit_vkc(vkc: winit::event::VirtualKeyCode) -> Button {
    unsafe {
        let code = std::mem::transmute::<
            winit::event::VirtualKeyCode,
            u32,
        >(vkc);
        std::mem::transmute::<u32, crate::Button>(code)
    }
}

// Convert a gilrs Button to an input button
// This is faillible since Gilrs can give us an Unknown button code
pub fn from_gilrs_button(button: gilrs::Button) -> Option<Button> {
    if matches!(button, gilrs::Button::Unknown) {
        return None;
    }

    unsafe {
        let mut code =
            std::mem::transmute::<gilrs::Button, u16>(button) as u32;
        code += OFFSET;
        Some(std::mem::transmute::<u32, crate::Button>(code))
    }
}

// The virtual keycodes that the window will receive (as a form of events)
// These will also sometimes represent buttons that are pressed by gamepads
// Stolen directly from the winit source code
#[derive(
    Serialize,
    Deserialize,
    Debug,
    Hash,
    Ord,
    PartialOrd,
    PartialEq,
    Eq,
    Clone,
    Copy,
)]
#[repr(u32)]
pub enum Button {
    /// The '1' key over the letters.
    Key1 = 0,
    /// The '2' key over the letters.
    Key2,
    /// The '3' key over the letters.
    Key3,
    /// The '4' key over the letters.
    Key4,
    /// The '5' key over the letters.
    Key5,
    /// The '6' key over the letters.
    Key6,
    /// The '7' key over the letters.
    Key7,
    /// The '8' key over the letters.
    Key8,
    /// The '9' key over the letters.
    Key9,
    /// The '0' key over the 'O' and 'P' keys.
    Key0,

    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    /// The Escape key, next to F1.
    Escape,

    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,

    /// Print Screen/SysRq.
    Snapshot,
    /// Scroll Lock.
    Scroll,
    /// Pause/Break key, next to Scroll lock.
    Pause,

    /// `Insert`, next to Backspace.
    Insert,
    Home,
    Delete,
    End,
    PageDown,
    PageUp,

    Left,
    Up,
    Right,
    Down,

    /// The Backspace key, right over Enter.
    // TODO: rename
    Back,
    /// The Enter key.
    Return,
    /// The space bar.
    Space,

    /// The "Compose" key on Linux.
    Compose,

    Caret,

    Numlock,
    Numpad0,
    Numpad1,
    Numpad2,
    Numpad3,
    Numpad4,
    Numpad5,
    Numpad6,
    Numpad7,
    Numpad8,
    Numpad9,
    NumpadAdd,
    NumpadDivide,
    NumpadDecimal,
    NumpadComma,
    NumpadEnter,
    NumpadEquals,
    NumpadMultiply,
    NumpadSubtract,

    AbntC1,
    AbntC2,
    Apostrophe,
    Apps,
    Asterisk,
    At,
    Ax,
    Backslash,
    Calculator,
    Capital,
    Colon,
    Comma,
    Convert,
    Equals,
    Grave,
    Kana,
    Kanji,
    LAlt,
    LBracket,
    LControl,
    LShift,
    LWin,
    Mail,
    MediaSelect,
    MediaStop,
    Minus,
    Mute,
    MyComputer,
    // also called "Next"
    NavigateForward,
    // also called "Prior"
    NavigateBackward,
    NextTrack,
    NoConvert,
    OEM102,
    Period,
    PlayPause,
    Plus,
    Power,
    PrevTrack,
    RAlt,
    RBracket,
    RControl,
    RShift,
    RWin,
    Semicolon,
    Slash,
    Sleep,
    Stop,
    Sysrq,
    Tab,
    Underline,
    Unlabeled,
    VolumeDown,
    VolumeUp,
    Wake,
    WebBack,
    WebFavorites,
    WebForward,
    WebHome,
    WebRefresh,
    WebSearch,
    WebStop,
    Yen,
    Copy,
    Paste,
    Cut,

    // Gamepad support from gilrs
    // Also copied from gilrs source code
    // Action Pad
    GamePadSouth = BTN_SOUTH,
    GamePadEast = BTN_EAST,
    GamePadNorth = BTN_NORTH,
    GamePadWest = BTN_WEST,
    GamePadC = BTN_C,
    GamePadZ = BTN_Z,

    // Triggers
    GamePadLeftTrigger = BTN_LT,
    GamePadLeftTrigger2 = BTN_LT2,
    GamePadRightTrigger = BTN_RT,
    GamePadRightTrigger2 = BTN_RT2,

    // Menu pad
    GamePadSelect = BTN_SELECT,
    GamePadStart = BTN_START,
    GamePadMode = BTN_MODE,

    // Sticks
    GamePadLeftThumb = BTN_LTHUMB,
    GamePadRightThumb = BTN_RTHUMB,

    // D-Pad
    GamePadDPadUp = BTN_DPAD_UP,
    GamePadDPadDown = BTN_DPAD_DOWN,
    GamePadDPadLeft = BTN_DPAD_LEFT,
    GamePadDPadRight = BTN_DPAD_RIGHT,
}

// GamePad mappings from gilrs source code
pub const OFFSET: u32 = 0xA2;

// Button code mappings
pub const BTN_SOUTH: u32 = 1 + OFFSET;
pub const BTN_EAST: u32 = 2 + OFFSET;
pub const BTN_C: u32 = 3 + OFFSET;
pub const BTN_NORTH: u32 = 4 + OFFSET;
pub const BTN_WEST: u32 = 5 + OFFSET;
pub const BTN_Z: u32 = 6 + OFFSET;
pub const BTN_LT: u32 = 7 + OFFSET;
pub const BTN_RT: u32 = 8 + OFFSET;
pub const BTN_LT2: u32 = 9 + OFFSET;
pub const BTN_RT2: u32 = 10 + OFFSET;
pub const BTN_SELECT: u32 = 11 + OFFSET;
pub const BTN_START: u32 = 12 + OFFSET;
pub const BTN_MODE: u32 = 13 + OFFSET;
pub const BTN_LTHUMB: u32 = 14 + OFFSET;
pub const BTN_RTHUMB: u32 = 15 + OFFSET;

// Dpad code mappings
pub const BTN_DPAD_UP: u32 = 16 + OFFSET;
pub const BTN_DPAD_DOWN: u32 = 17 + OFFSET;
pub const BTN_DPAD_LEFT: u32 = 18 + OFFSET;
pub const BTN_DPAD_RIGHT: u32 = 19 + OFFSET;
