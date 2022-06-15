use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::new().insert_startup(init).insert_update(update).execute();
}

// Initialize the empty world
fn init(_world: &mut World) {
    println!("Start!")
}

// Le update
fn update(world: &mut World) {
    /*
    let time = world.get_mut::<&mut Time>().unwrap();
    dbg!(1.0 / time.delta());
    */
}