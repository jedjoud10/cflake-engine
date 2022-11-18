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
    input.bind_key("forward", Key::W);
    input.bind_key("backward", Key::S);
}

// Read from the mappings
fn update(world: &mut World) {
    let input = world.get::<Input>().unwrap();

    // Print out the "backward" message if we press the "W" key
    if input.key("forward").pressed() {
        println!("Going forward!");
    }

    // Print out the "backward" message if we press the "S" key
    if input.key("backward").pressed() {
        println!("Going backward!");
    }
}