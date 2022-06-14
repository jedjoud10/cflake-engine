use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::new().insert_startup(init).execute();
}

// Initialize the empty world, and try to load a default asset
// This will try to load the default vertex source for the PBR shader
fn init(world: &mut World) {
    let loader = world.get_mut::<&mut Assets>().unwrap();
    loader.load(path) 
}