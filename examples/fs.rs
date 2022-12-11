use cflake_engine::prelude::*;
use std::io::Write;

// An empty game window
fn main() {
    App::default()
        .insert_init(init)
        .set_window_title("Hello World!")
        .execute();
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "cflake_engine::prelude::serde")]
struct SimpleStruct {
    name: String,
    value: u32,
}

// First function that gets executed when the engine starts
fn init(world: &mut World) {
    let mut fm = world.get_mut::<FileManager>().unwrap();

    // Write to the config JSON file
    fm.serialize(
        &SimpleStruct {
            name: "Test name".to_owned(),
            value: 50,
        },
        "config.json",
    )
    .unwrap();
}
