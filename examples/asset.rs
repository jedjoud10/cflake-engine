use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::new().insert_startup(init).execute();
}

// Initialize the empty world, and try to load a default asset
// This will try to load the default vertex source for the PBR shader
fn init(world: &mut World) {
    let assets = world.get_mut::<&mut Assets>().unwrap();
    let text = assets.load::<String>("assets/defaults/test.txt").unwrap();
}