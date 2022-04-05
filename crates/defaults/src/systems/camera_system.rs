use crate::components::{Camera, Transform};
use crate::globals::GlobalWorldData;
use world::ecs::{BundleData, Entity, Query};
use world::World;
// The camera system update loop
fn run(world: &mut World) {
    // Set the main camera entity key in the world global
    let global = world.globals.get_mut::<GlobalWorldData>().unwrap();
    // If there isn't a main camera assigned already, we can be the main one
    if global.camera == Entity::default() {
        // Query all the cameras in the world and get the first one
        let query = Query::<(&Transform, &Camera, &BundleData)>::new(&mut world.ecs).unwrap();

        // And try to get the first valid one
        if let Some((_, _, data)) = query.fetch().unwrap().first() {
            global.camera = data.entity();
        }
    }
}
// Create the camera system
pub fn system(world: &mut World) {
    world.systems.insert(run);
}
