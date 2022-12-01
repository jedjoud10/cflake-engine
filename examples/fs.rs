use cflake_engine::prelude::*;
use std::io::Write;

// An empty game window
fn main() {
    App::default()
        .insert_init(init)
        .set_window_title("Hello World!")
        .execute();
}

// First function that gets executed when the engine starts
fn init(world: &mut World) {
    let filemanager = world.get_mut::<FileManager>().unwrap();
    let mut writer = filemanager.write("config.txt").unwrap();
    writeln!(writer, "Test").unwrap();
}
