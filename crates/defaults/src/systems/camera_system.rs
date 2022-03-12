use world::ecs::component::{ComponentQueryParameters, ComponentQuerySet};

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

    // Update the camera values now
    let components = query.all.get_mut(&global.main_camera);
    if let Some(components) = components {
        // Get the linked components
        let (position, rotation, mutated) = {
            let transform = components.get::<Transform>().unwrap();
            (transform.position, transform.rotation, components.was_mutated::<Transform>().unwrap())
        };
        let camera = components.get_mut::<Camera>().unwrap();

        // Calculate aspect ratio
        let ratio = world.pipeline.window.dimensions().x as f32 / world.pipeline.window.dimensions().y as f32;

        // And don't forget to update the camera matrices
        camera.update_projection_matrix(ratio);
        if mutated {
            camera.update_view_matrix(position, rotation);
        }
    }
}

// Create the camera system
pub fn system(world: &mut World) {
    world
        .ecs
        .systems
        .builder()
        .event(run)
        .query(ComponentQueryParameters::default().link::<Camera>().link::<Transform>())
        .build();
}
