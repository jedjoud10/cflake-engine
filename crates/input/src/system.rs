//use world::{post_user, user, System, WindowEvent, World};

/*
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
    fn handle_button_input(input: &mut Input, key: Button, state: ElementState) {
        match input.keys.entry(key) {
            Entry::Occupied(mut current) => {
                // Check if the key is "down" (either pressed or held)
                let down = matches!(*current.get(), ButtonState::Pressed | ButtonState::Held);

                // If the key is pressed while it is currently down, it repeated itself, and we must ignore it
                if down ^ (state == ElementState::Pressed) {
                    current.insert(state.into());
                }
            }
            Entry::Vacant(v) => {
                v.insert(state.into());
            }
        }
    }

    let mut input = world.get_mut::<Input>().unwrap();

    match ev {
        // Handles keyboard keys
        WindowEvent::KeyboardInput { input: key, .. } => {
            if let Some(keycode) = key.virtual_keycode {
                handle_button_input(&mut input, Button::Keyboard(keycode), key.state);
            }
        }

        // Handles mouse buttons
        WindowEvent::MouseInput { state, button, .. } => {
            handle_button_input(&mut input, Button::Mouse(*button), *state);
        }

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
            input.axii.insert(Axis::Mouse(MouseAxis::DeltaX), delta.x);
            input.axii.insert(Axis::Mouse(MouseAxis::DeltaY), delta.y);
            let x = input
                .axii
                .entry(Axis::Mouse(MouseAxis::PositionX))
                .or_insert(0.0);
            *x += delta.x;
            let y = input
                .axii
                .entry(Axis::Mouse(MouseAxis::PositionY))
                .or_insert(0.0);
            *y += delta.y;
        }

        // Update mouse wheel delta and summed value
        DeviceEvent::MouseWheel { delta } => {
            let delta = match delta {
                winit::event::MouseScrollDelta::LineDelta(_, y) => *y,
                winit::event::MouseScrollDelta::PixelDelta(physical) => physical.x as f32,
            };

            input
                .axii
                .insert(Axis::Mouse(MouseAxis::ScrollDelta), delta);
            let scroll = input
                .axii
                .entry(Axis::Mouse(MouseAxis::Scroll))
                .or_insert(0.0);
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
    if let Some(data) = input.axii.get_mut(&Axis::Mouse(MouseAxis::ScrollDelta)) {
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
                if button != gilrs::Button::Unknown {
                    let state = input
                        .keys
                        .entry(Button::Gamepad(button))
                        .or_insert(ButtonState::Pressed);
                    *state = ButtonState::Pressed;
                }
            }

            // Button released event
            gilrs::EventType::ButtonReleased(button, _) => {
                if button != gilrs::Button::Unknown {
                    let state = input
                        .keys
                        .entry(Button::Gamepad(button))
                        .or_insert(ButtonState::Released);
                    *state = ButtonState::Released;
                }
            }

            // Axis changed event
            gilrs::EventType::AxisChanged(axis, value, _) => {
                if axis != gilrs::Axis::Unknown {
                    input.axii.insert(Axis::Gamepad(axis), value);
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
*/