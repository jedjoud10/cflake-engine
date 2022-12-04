use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::default()
        .insert_init(init)
        .insert_update(update)
        .execute();
}

// Map some input buttons
fn init(world: &mut World) {
    let mut input = world.get_mut::<Input>().unwrap();
    input.bind_button("forward", Button::GamePadDPadUp);
    input.bind_button("backward", Button::GamePadDPadDown);
}

// Read from the mappings
fn update(world: &mut World) {
    let input = world.get::<Input>().unwrap();

    // Print out the "backward" message if we press the "W" key
    if input.get_button("forward").pressed() {
        println!("Going forward!");
    }

    // Print out the "backward" message if we press the "S" key
    if input.get_button("backward").pressed() {
        println!("Going backward!");
    }

    if input.get_axis(Axis::MousePositionDeltaX) != 0.0 {
        println!("{}", input.get_axis(Axis::MousePositionDeltaX));
    }
}
