use crate::{components::Transform, globals::GlobalWorldData};
use world::World;

// Update the position of the left and right ears
fn run(world: &mut World) {
    // Global
    let global = world.globals.get::<GlobalWorldData>().unwrap();
    let entry = world.ecs.entry(global.main_camera);
    if let Some(entry) = entry {
        // Get the component
        let transform = entry.get::<Transform>().unwrap();

        // Update the positions
        let pos = transform.position;
        let right = transform.right();
        world.audio.update(pos - right, pos + right);
    }
}

// Create the audio system
pub fn system(world: &mut World) {
    world.systems.insert(run);
}
