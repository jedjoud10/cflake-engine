use cflake_engine::prelude::*;
use std::io::Write;

// An empty game window
fn main() {
    App::default()
        .insert_init(init)
        .set_app_name("Hello World!")
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
    fm.serialize_into_file(
        &SimpleStruct {
            name: "Test name".to_owned(),
            value: 50,
        },
        "config.json",
        FileType::Config
    )
    .unwrap();
}
