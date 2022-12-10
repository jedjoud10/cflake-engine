use cflake_engine::prelude::*;

// Prototype example game window
fn main() {
    App::default()
        .set_window_title("cflake engine prototype example")
        .insert_update(update)
        .insert_init(init)
        .execute();
}

// Executed at the start
fn init(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap().clone();
    let mut threadpool = world.get_mut::<ThreadPool>().unwrap();
}

// Executed each frame
fn update(world: &mut World) {
    let input = world.get::<Input>().unwrap();
    let time = world.get::<Time>().unwrap();

    if input.get_button(Button::P).pressed() {
        println!("{}", time.average_fps());
    }
}
