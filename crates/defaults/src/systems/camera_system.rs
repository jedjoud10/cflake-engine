use crate::components::{Camera, Transform};
use crate::resources::WorldData;
use world::ecs::Entity;
use world::World;
// The camera system update loop
fn run(world: &mut World) {
    // Set the main camera entity key in the world global
    let global = world.resources.get_mut::<WorldData>().unwrap();
    // If there isn't a main camera assigned already, we can be the main one
    if global.camera == Entity::default() {
        // Query all the cameras in the world and get the first one
        let mut query = world.ecs.try_query::<(&Camera, &Transform, &Entity)>();
        //query.next();
        // And try to get the first valid one
        if let Some((_, _, entity)) = query.next() {
            global.camera = *entity;
        }
    }
}
// Create the camera system
pub fn system(world: &mut World) {
    world.events.insert(run);
}
