use crate::components::{Camera, Transform};
use crate::globals::GlobalWorldData;
use world::World;
/*
// The camera system update loop
fn run(world: &mut World) {
    // Set the main camera entity key in the world global
    let global = world.globals.get_mut::<GlobalWorldData>().unwrap();
    // If there isn't a main camera assigned already, we can be the main one
    if let Some((&key, _)) = query.delta.added.iter().next() {
        global.main_camera = key;
    }
}

// Create the camera system
pub fn system(world: &mut World) {
    world
        .events
        .insert(run())
        .system()
        .systems
        .builder(&mut world.events.ecs)
        .event(run)
        .query(ComponentQueryParams::default().link::<Camera>().link::<Transform>())
        .build()
        .unwrap();
}
*/