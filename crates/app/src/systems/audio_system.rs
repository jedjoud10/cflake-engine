use crate::{components::Transform, resources::WorldData};
use world::World;

// Update the position of the left and right ears
fn run(world: &mut World) {
    // Global
    let global = world.resources.get::<WorldData>().unwrap();
    let entry = world.ecs.entry(global.camera);
    if let Some(entry) = entry {
        if !entry.was_mutated::<Transform>().unwrap() {
            return;
        }
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
    world.events.insert(run);
}