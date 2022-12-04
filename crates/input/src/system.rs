use std::collections::hash_map::Entry;

use crate::{Axis, ButtonState, Input};
use winit::event::{DeviceEvent, ElementState};
use world::{post_user, user, System, World};

// Init event (called once at the start of program)
fn init(world: &mut World) {
    world.insert(Input {
        bindings: Default::default(),
        keys: Default::default(),
        axii: Default::default(),
        gilrs: gilrs::Gilrs::new().unwrap(),

        #[cfg(feature = "sentence-recording")]
        sentence: None,

        #[cfg(feature = "sentence-recording")]
        sentence_nl_action: crate::NewLineAction::Clear,
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
                let button = crate::from_winit_vkc(keycode);
                match input.keys.entry(button) {
                    Entry::Occupied(mut current) => {
                        // Check if the key is "down" (either pressed or held)
                        let down = matches!(
                            *current.get(),
                            ButtonState::Pressed | ButtonState::Held
                        );

                        // If the key is pressed while it is currently down, it repeated itself, and we must ignore it
                        if down ^ (key.state == ElementState::Pressed) {
                            current.insert(key.state.into());
                        }
                    }
                    Entry::Vacant(v) => {
                        v.insert(key.state.into());
                    }
                }

                // Only used for sentence recording
                #[cfg(feature = "sentence-recording")]
                {
                    let action = input.sentence_nl_action;
                    if let Some(sentence) = &mut input.sentence {
                        if matches!(
                            keycode,
                            winit::event::VirtualKeyCode::Back
                        ) {
                            sentence.pop();
                        } else if matches!(
                            keycode,
                            winit::event::VirtualKeyCode::Return
                        ) {
                            match action {
                                crate::NewLineAction::Clear => {
                                    sentence.clear()
                                }
                                crate::NewLineAction::NewLine => {
                                    sentence.push('\n')
                                }
                            }
                        }
                    }
                }
            }
        }

        _ => {}
    }
}

// Winit window event (called by handler when needed)
#[cfg(feature = "sentence-recording")]
fn window(world: &mut World, ev: &mut WindowEvent) {
    if let WindowEvent::ReceivedCharacter(char) = ev {
        let mut input = world.get_mut::<Input>().unwrap();
        if let Some(sentence) = &mut input.sentence {
            sentence.push(*char);
        }
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
                if let Some(button) = crate::from_gilrs_button(button)
                {
                    let state = input
                        .keys
                        .entry(button)
                        .or_insert(ButtonState::Pressed);
                    *state = ButtonState::Pressed;
                }
            }

            // Button released event
            gilrs::EventType::ButtonReleased(button, _) => {
                if let Some(button) = crate::from_gilrs_button(button)
                {
                    let state = input
                        .keys
                        .entry(button)
                        .or_insert(ButtonState::Released);
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
    system.insert_device(event).before(user);
    system.insert_update(update).after(post_user);

    #[cfg(feature = "sentence-recording")]
    system.insert_window(window).before(user);
}
