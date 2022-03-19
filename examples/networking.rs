use cflake_engine::{*, defaults::systems::networking_system, ecs::component::ComponentQuerySet};

// An empty game window
fn main() {
    cflake_engine::start("cflake-examples", "networking", init, |world| {
        networking_system::debugging_system::system(world);

        // Messenger
        world.ecs.systems.builder(&mut world.events.ecs).event(run).build().unwrap();
    })
}

// Create a system that will send / receive messages
fn run(world: &mut World, data: ComponentQuerySet) {
}


// Init the empty world
fn init(_world: &mut World) {}
