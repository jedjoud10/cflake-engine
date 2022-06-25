use ahash::AHashMap;
use glutin::event::ElementState;
use world::{Resource, World, Update, DeviceEvent};

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
    fn pressed(&self) -> bool {
        match self {
            State::Pressed => true,
            _ => false,
        }
    }

    // This checks if the state is equal to State::Released
    fn released(&self) -> bool {
        match self {
            State::Released => true,
            _ => false,
        }
    }

    // This checks if the State is equal to State::Held
    fn held(&self) -> bool {
        match self {
            State::Held => todo!(),
            _ => false,
        }
    }
}

// An input manager takes keyboard and mouse inputs from the Glutin window and maps them to specific key binds
#[derive(Default)]
pub struct Input {
    // Keyboard
    // "forward_key_bind" -> Key::W
    binds: AHashMap<&'static str, Key>,

    // Key::W -> State::Pressed
    keys: AHashMap<Key, State>,

    // Mouse position and mouse scroll values
    position: vek::Vec2<f32>,
    scroll: f32,
}

impl Resource for Input {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn removable(world: &mut World) -> bool where Self: Sized {
        false
    }

    fn inserted(&mut self, world: &mut World) {
        // Late update event
        world.events().register_with::<Update>(|world: &mut World| {
            // Convert all the State::Pressed keys to State::Held and all the State::Released to State::None
            let input = world.get_mut::<&mut Input>().unwrap();
            for (_, state) in input.keys.iter_mut() {
                *state = match state {
                    State::Pressed => State::Held,
                    State::Released => State::None,
                    State::Held => State::Held,
                    State::None => State::None,
                };
            }
        }, i32::MAX);
        
        // Device input event for keyboard / mouse input
        world.events().register::<DeviceEvent>(|world: &mut World, event: &glutin::event::DeviceEvent| {
            let res = world.get_mut::<&mut Input>().unwrap();
            match event {
                glutin::event::DeviceEvent::MouseMotion { delta } => {
                    // Update the mouse position delta value

                },
                glutin::event::DeviceEvent::MouseWheel { delta } => {
                    // Update the mouse wheel delta value
                    
                },
                glutin::event::DeviceEvent::Key(input) => {
                    // Update the state of the pressed key (only if it is valid)
                    if let Some(keycode) = input.virtual_keycode {
                        *res.keys.entry(keycode).or_insert(State::Pressed) = match input.state {
                            ElementState::Pressed => State::Pressed,
                            ElementState::Released => State::Released,
                        };
                    }
                },
                _ => {}
            }
        });
    }
}

impl Input {
    // Create a new key binding using a name and a unique key
    pub fn bind(&mut self, name: &'static str, key: Key) {
        self.binds.insert(name, key);
    }

    // Get the raw state of a key bind (map)
    pub fn state(&self, name: &'static str) -> Option<&State> {
        self.binds.get(name).and_then(|key| self.keys.get(key))
    }

    // Check if a keybind was pressed in the current frame
    pub fn pressed(&self, name: &'static str) -> bool {
        self.state(name).map(State::pressed).unwrap_or_default()
    }

    // Check if a keybind is being held (a held key is just a key that has been pressed for more than 2 frames)
    pub fn held(&self, name: &'static str) -> bool {
        self.state(name).map(State::held).unwrap_or_default()
    }

    // Check if a keybind was released in the current frame
    pub fn released(&self, name: &'static str) -> bool {
        self.state(name).map(State::released).unwrap_or_default()
    }

    // Get the current mouse position delta
    pub fn mouse()
}
