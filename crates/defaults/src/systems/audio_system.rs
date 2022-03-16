use world::ecs::component::{ComponentQueryParams, ComponentQuerySet};
use world::World;

use crate::components::{Camera, Transform};
use crate::globals::GlobalWorldData;

// Update the position of the left and right ears
fn run(world: &mut World, data: ComponentQuerySet) {
    // Global
    let global = world.globals.get::<GlobalWorldData>().unwrap();
    let components = data.get(0).unwrap().all.get(&global.main_camera);
    if let Some(components) = components {
        let transform = components.get::<Transform>().unwrap();
        let pos = transform.position;
        let right = transform.right();
        // Update the positions
        world.audio.update(pos - right, pos + right);
    }
}

// Create the audio system
pub fn system(world: &mut World) {
    world
        .ecs
        .systems
        .builder(&mut world.events.ecs)
        .event(run)
        .query(ComponentQueryParams::default().link::<Camera>().link::<Transform>())
        .build()
        .unwrap();
}
