use crate::components::{Camera, Transform};
use crate::globals::GlobalWorldData;
use world::{World};
use world::ecs::{Entity, QueryBuilder, layout};
// The camera system update loop
fn run(world: &mut World) {
    // Set the main camera entity key in the world global
    let global = world.globals.get_mut::<GlobalWorldData>().unwrap();
    // If there isn't a main camera assigned already, we can be the main one
    if global.camera == Entity::default() {
        // Query
        let builder = QueryBuilder::new(&mut world.ecs, layout!(Transform, Camera));

        // Fetch the entity handles
        let cameras = builder.get::<Entity>().unwrap();

        // And try to get the first valid one
        if let Some(entity) = cameras.first() {
            global.camera = *entity;
        }
    }
}
// Create the camera system
pub fn system(world: &mut World) {
    world.systems.insert(run);
}
