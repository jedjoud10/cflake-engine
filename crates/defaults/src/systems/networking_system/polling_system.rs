use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use world::{
    ecs::component::ComponentQuerySet,
    gui::egui,
    network::{register, Client, Host, NetworkSession},
    World,
};

use crate::globals::NetworkManager;

// Update the network manager (if we have one)
fn run(world: &mut World, mut _data: ComponentQuerySet) {
    // Simple GUI
    let manager = world.globals.get_mut::<NetworkManager>().unwrap();
    if let Some(session) = &mut manager.session {
        session.update().unwrap();
    }
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
    world.ecs.systems.builder(&mut world.events.ecs).event(run).build().unwrap();
}
