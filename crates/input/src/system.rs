use crate::{Axis, Input, KeyState};
use winit::event::{DeviceEvent, ElementState};
use world::{Events, Init, Stage, Update, World};

// This system will automatically insert the input resource and update it each frame using the window events
pub fn system(events: &mut Events) {
    // Init event (called once at the start of program)
    fn init(world: &mut World) {
        world.insert(Input {
            bindings: Default::default(),
            keys: Default::default(),
            axii: Default::default(),
        });
    }

    // Glutin window event (called by handler when needed)
    fn event(world: &mut World, ev: &DeviceEvent) {
        let mut input = world.get_mut::<Input>().unwrap();
        match ev {
            // Update mouse position delta and summed  pos
            DeviceEvent::MouseMotion { delta } => {
                let delta = vek::Vec2::<f64>::from(*delta).as_::<f32>();
                input.axii.insert(Axis::MousePositionDeltaX, delta.x);
                input.axii.insert(Axis::MousePositionDeltaY, delta.y);
                let x = input.axii.entry(Axis::MousePositionX).or_insert(0.0);
                *x += delta.x;
                let y = input.axii.entry(Axis::MousePositionY).or_insert(0.0);
                *y += delta.y;
            }

            // Update mouse wheel delta and summed value
            DeviceEvent::MouseWheel { delta } => {
                let delta = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
                    winit::event::MouseScrollDelta::PixelDelta(physical) => physical.x as f32,
                };

                input.axii.insert(Axis::MouseScrollDelta, delta);
                let scroll = input.axii.entry(Axis::MouseScroll).or_insert(0.0);
                *scroll += delta;
            }

            // Update keyboard key states
            DeviceEvent::Key(key) => {
                if let Some(keycode) = key.virtual_keycode {
                    match input.keys.entry(keycode) {
                        std::collections::hash_map::Entry::Occupied(mut current) => {
                            // Check if the key is "down" (either pressed or held)
                            let down = match *current.get() {
                                KeyState::Pressed | KeyState::Held => true,
                                _ => false,
                            };

                            // If the key is pressed while it is currently down, it repeated itself, and we must ignore it
                            if down ^ (key.state == ElementState::Pressed) {
                                current.insert(key.state.into());
                            }
                        }
                        std::collections::hash_map::Entry::Vacant(v) => {
                            v.insert(key.state.into());
                        }
                    }
                }
            }

            _ => {}
        }
    }

    // Update event that will change the state of the keyboard keys (some states are sticky while others are not sticky)
    fn update(world: &mut World) {
        let mut keyboard = world.get_mut::<Input>().unwrap();
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
        .registry_mut::<Init>()
        .insert_with(init, Stage::new("input insert").before("user"))
        .unwrap();
    events
        .registry_mut::<DeviceEvent>()
        .insert_with(event, Stage::new("input").before("user"))
        .unwrap();
    events
        .registry_mut::<Update>()
        .insert_with(
            update,
            Stage::new("keyboard update states").after("post user"),
        )
        .unwrap();
}
