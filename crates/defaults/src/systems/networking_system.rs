use world::{network::register, World};
use crate::globals::NetworkManager;

// Update the network manager (if we have one)
fn run(world: &mut World) {
    let global = world.globals.get_mut::<NetworkManager>().unwrap();
    global.session.update().unwrap();
}

// Create the networking system
pub fn system(world: &mut World) {
    // Register some common types
    register::<f32>();
    register::<f64>();
    register::<u8>();
    register::<u16>();
    register::<u32>();
    register::<u64>();
    register::<u128>();
    register::<i8>();
    register::<i16>();
    register::<i32>();
    register::<i64>();
    register::<i128>();
    register::<String>();

    world.systems.insert(run);
}
