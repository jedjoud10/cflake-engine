use world::ecs::component::{ComponentQueryParams, ComponentQuerySet};


use world::World;

use crate::components::{Camera, Transform};
use crate::globals::GlobalWorldData;

// The camera system update loop
fn run(world: &mut World, mut data: ComponentQuerySet) {
    let query = data.get_mut(0).unwrap();

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
        .ecs
        .systems
        .builder()
        .event(run)
        .query(ComponentQueryParams::default().link::<Camera>().link::<Transform>())
        .build();
}
