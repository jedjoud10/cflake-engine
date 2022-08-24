use crate::{KeyState, Keyboard, Mouse};
use glutin::event::{DeviceEvent, ElementState};
use world::{Events, Init, Stage, Update, World};

// This system will automatically insert the input resource and update it each frame using the window events
pub fn system(events: &mut Events) {
    // Init event (called once at the start of program)
    fn init(world: &mut World) {
        world.insert(Keyboard {
            binds: Default::default(),
            keys: Default::default(),
        });

        world.insert(Mouse {
            scroll_delta: 0.0,
            scroll: 0.0,
            pos_delta: Default::default(),
            pos: Default::default(),
        });
    }

    // Glutin window event (called by handler when needed)
    fn event(world: &mut World, ev: &DeviceEvent) {
        let mut keyboard = world.get_mut::<Keyboard>().unwrap();
        let mut mouse = world.get_mut::<Mouse>().unwrap();

        match ev {
            // Update mouse position delta and summed  pos
            DeviceEvent::MouseMotion { delta } => {
                let delta = vek::Vec2::<f64>::from(*delta).as_::<f32>();
                mouse.pos += delta;
                mouse.pos_delta = delta;
            }

            // Update mouse wheel delta and summed value
            DeviceEvent::MouseWheel { delta } => {
                let delta = match delta {
                    glutin::event::MouseScrollDelta::LineDelta(_, y) => *y,
                    glutin::event::MouseScrollDelta::PixelDelta(physical) => physical.x as f32,
                };
                mouse.scroll += delta;
                mouse.scroll_delta = delta;
            }

            // Update keyboard key states
            DeviceEvent::Key(key) => {
                if let Some(keycode) = key.virtual_keycode {
                    match keyboard.keys.entry(keycode) {
                        std::collections::hash_map::Entry::Occupied(mut current) => {
                            // Check if the key is "down" (either pressed or held)
                            let down = match *current.get() {
                                KeyState::Pressed | KeyState::Held => true,
                                _ => false
                            };
                            
                            // If the key is pressed while it is currently down, it repeated itself, and we must ignore it                       
                            if down ^ (key.state == ElementState::Pressed) {
                                current.insert(key.state.into());
                            }
                        },
                        std::collections::hash_map::Entry::Vacant(v) => { v.insert(key.state.into()); },
                    }
                }
            }

            _ => {}
        }
    }

    // Update event that will change the state of the keyboard keys (some states are sticky while others are not sticky)
    fn update(world: &mut World) {
        let mut keyboard = world.get_mut::<Keyboard>().unwrap();
        for (_, state) in keyboard.keys.iter_mut() {
            *state = match state {
                crate::KeyState::Pressed => KeyState::Held,
                crate::KeyState::Released => KeyState::None,
                crate::KeyState::Held => KeyState::Held,
                crate::KeyState::None => KeyState::None,
            };
        }
    }

    // Register the events
    events
        .registry::<Init>()
        .insert_with(init, Stage::new("input insert").before("user"))
        .unwrap();
    events
        .registry::<DeviceEvent>()
        .insert_with(event, Stage::new("input").before("user"))
        .unwrap();
    events
        .registry::<Update>()
        .insert_with(
            update,
            Stage::new("keyboard update states").after("post user"),
        )
        .unwrap();
}
