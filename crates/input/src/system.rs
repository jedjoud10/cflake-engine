use crate::{Axis, Input, ButtonState};
use winit::event::{DeviceEvent, ElementState};
use world::{post_user, user, System, World};

// Init event (called once at the start of program)
fn init(world: &mut World) {
    world.insert(Input {
        bindings: Default::default(),
        keys: Default::default(),
        axii: Default::default(),
        gilrs: gilrs::Gilrs::new().unwrap(),
        gamepad: None,
    });
}

// Winit device event (called by handler when needed)
fn event(world: &mut World, ev: &DeviceEvent) {
    let mut input = world.get_mut::<Input>().unwrap();
    match ev {
        // Update mouse position delta and summed  pos
        DeviceEvent::MouseMotion { delta } => {
            let delta = vek::Vec2::<f64>::from(*delta).as_::<f32>();
            input.axii.insert(Axis::MousePositionDeltaX, delta.x);
            input.axii.insert(Axis::MousePositionDeltaY, delta.y);
            let x =
                input.axii.entry(Axis::MousePositionX).or_insert(0.0);
            *x += delta.x;
            let y =
                input.axii.entry(Axis::MousePositionY).or_insert(0.0);
            *y += delta.y;
        }

        // Update mouse wheel delta and summed value
        DeviceEvent::MouseWheel { delta } => {
            let delta = match delta {
                winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
                winit::event::MouseScrollDelta::PixelDelta(
                    physical,
                ) => physical.x as f32,
            };

            input.axii.insert(Axis::MouseScrollDelta, delta);
            let scroll =
                input.axii.entry(Axis::MouseScroll).or_insert(0.0);
            *scroll += delta;
        }

        // Update keyboard key states
        DeviceEvent::Key(key) => {
            if let Some(keycode) = key.virtual_keycode {
                // Sorry :(
                let keycode = unsafe {
                    let id = std::mem::transmute::<winit::event::VirtualKeyCode, u32>(keycode);
                    std::mem::transmute::<u32, crate::Button>(id)
                };

                match input.keys.entry(keycode) {
                    std::collections::hash_map::Entry::Occupied(
                        mut current,
                    ) => {
                        // Check if the key is "down" (either pressed or held)
                        let down = matches!(
                            *current.get(),
                            ButtonState::Pressed | ButtonState::Held
                        );

                        // If the key is pressed while it is currently down, it repeated itself, and we must ignore it
                        if down ^ (key.state == ElementState::Pressed)
                        {
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
// This will also read the state from gamepads using gilrs
fn update(world: &mut World) {
    let mut input = world.get_mut::<Input>().unwrap();
    for (_, state) in input.keys.iter_mut() {
        *state = match state {
            crate::ButtonState::Pressed => ButtonState::Held,
            crate::ButtonState::Released => ButtonState::None,
            crate::ButtonState::Held => ButtonState::Held,
            crate::ButtonState::None => ButtonState::None,
        };
    }


    // Set the current gamepad if it's not set yet
    if let Some((new, _)) = input.gilrs.gamepads().next() {
        input.gamepad.get_or_insert(new);
    }

    // Update the gamepad axii and buttson
    if let Some(id) = input.gamepad {
        if let Some(gamepad) = input.gilrs.connected_gamepad(id) {
            // TODO: This
        } else {
            input.gamepad = None;
        }
    } 

    // TODO: Write gamepad support using gilrs
}

// This system will automatically insert the input resource and update it each frame using the window events
pub fn system(system: &mut System) {
    system.insert_init(init).before(user);
    system.insert_device(event).before(user);
    system.insert_update(update).after(post_user);
}
