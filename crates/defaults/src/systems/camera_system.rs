use crate::components::{Camera, Transform};
use crate::globals::GlobalWorldData;
use world::ecs::Entity;
use world::World;
// The camera system update loop
fn run(world: &mut World) {
    // Set the main camera entity key in the world global
    let global = world.globals.get_mut::<GlobalWorldData>().unwrap();
    // If there isn't a main camera assigned already, we can be the main one
    if global.camera == Entity::default() {
        // Query all the cameras in the world and get the first one
        let mut query = world.ecs.query::<(&Transform, &Camera)>();

        // And try to get the first valid one
        // TODO: Fetch entity ID through query
        /*
        if let Some((_, _, entity)) = query.next() {
            global.camera = *entity;
        }
        */
    }
}
// Create the camera system
pub fn system(world: &mut World) {
    world.events.insert(run);
}
