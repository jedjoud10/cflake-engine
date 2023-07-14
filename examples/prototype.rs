use cflake_engine::prelude::*;

// Prototype example game window
fn main() {
    App::default()
        .set_app_name("cflake engine prototype example")
        .insert_update(update)
        .execute();
}

// Test update
fn update(world: &mut World) {
    let input = world.get::<Input>().unwrap();

    if input.get_button(KeyboardButton::K).pressed() {
        let time = world.get::<Time>().unwrap();
        println!("FPS: {}", 1.0 / time.delta().as_secs_f64());
    }
}
