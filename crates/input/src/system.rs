use std::collections::hash_map::Entry;

use crate::{Axis, ButtonState, Input};
use gilrs::PowerInfo;
use winit::event::{DeviceEvent, ElementState};
use world::{post_user, user, System, World, WindowEvent};

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

// Winit window event since it seems that DeviceEvent::Key is broken on other machines
// TODO: Report bug
fn window_event(world: &mut World, ev: &mut WindowEvent) {
    let mut input = world.get_mut::<Input>().unwrap();

    match ev {
        WindowEvent::KeyboardInput { input: key, .. } => {
            if let Some(keycode) = key.virtual_keycode {
                let button = crate::from_winit_vkc(keycode);
                match input.keys.entry(button) {
                    Entry::Occupied(mut current) => {
                        // Check if the key is "down" (either pressed or held)
                        let down =
                            matches!(*current.get(), ButtonState::Pressed | ButtonState::Held);

                        // If the key is pressed while it is currently down, it repeated itself, and we must ignore it
                        if down ^ (key.state == ElementState::Pressed) {
                            current.insert(key.state.into());
                        }
                    }
                    Entry::Vacant(v) => {
                        v.insert(key.state.into());
                    }
                }
            }
        },
        _ => {}
    }
}

// Winit device event (called by handler when needed)
fn device_event(world: &mut World, ev: &DeviceEvent) {
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

        _ => {}
    }
}

// Update event that will change the state of the keyboard keys (some states are sticky while others are not sticky)
// This will also read the state from gamepads using gilrs
fn update(world: &mut World) {
    let mut input = world.get_mut::<Input>().unwrap();

    // Update the state of the keys/buttons
    for (_, state) in input.keys.iter_mut() {
        *state = match state {
            crate::ButtonState::Pressed => ButtonState::Held,
            crate::ButtonState::Released => ButtonState::None,
            crate::ButtonState::Held => ButtonState::Held,
            crate::ButtonState::None => ButtonState::None,
        };
    }

    // Reset the mouse scroll delta (since winit doesn't reset it for us)
    if let Some(data) = input.axii.get_mut(&Axis::MouseScrollDelta) {
        *data = 0f32;
    }

    // Try to get the currently used gamepad
    let gamepad = input
        .gamepad
        .and_then(|main| input.gilrs.connected_gamepad(main));

    // Report battery level if critical
    if let Some(gamepad) = gamepad {
        let name = gamepad.name();
        let info = gamepad.power_info();

        match info {
            PowerInfo::Discharging(val) if val < 10 => {
                log::warn!("Gamepad {name} is reaching critical battery levels.")
            }
            _ => {}
        }
    }

    // Update the gamepad axii and buttson
    while let Some(event) = input.gilrs.next_event() {
        // Skip if we already have a main controller and it isn't it
        if input
            .gamepad
            .map(|main| main != event.id)
            .unwrap_or_default()
        {
            return;
        }

        match event.event {
            // Button pressed event
            gilrs::EventType::ButtonPressed(button, _) => {
                if let Some(button) = crate::from_gilrs_button(button) {
                    let state = input.keys.entry(button).or_insert(ButtonState::Pressed);
                    *state = ButtonState::Pressed;
                }
            }

            // Button released event
            gilrs::EventType::ButtonReleased(button, _) => {
                if let Some(button) = crate::from_gilrs_button(button) {
                    let state = input.keys.entry(button).or_insert(ButtonState::Released);
                    *state = ButtonState::Released;
                }
            }

            // Axis changed event
            gilrs::EventType::AxisChanged(axis, value, _) => {
                if let Some(axis) = crate::from_gilrs_axis(axis) {
                    input.axii.insert(axis, value);
                }
            }

            // Add the main gamepad controller
            gilrs::EventType::Connected => {
                input.gamepad.get_or_insert(event.id);
            }

            // Remove the main gamepad controller
            gilrs::EventType::Disconnected => input.gamepad = None,
            _ => (),
        }
    }
}

// This system will automatically insert the input resource and update it each frame using the window events
pub fn system(system: &mut System) {
    system.insert_init(init).before(user);
    system.insert_device(device_event).before(user);
    system.insert_window(window_event).before(user);
    system.insert_update(update).after(post_user);
}
