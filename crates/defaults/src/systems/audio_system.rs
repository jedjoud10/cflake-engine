use world::ecs::event::EventKey;
use world::World;

// Update the position of the left and right ears
fn run(world: &mut World, _data: EventKey) {
    // Global
    let global = world.globals.get::<crate::globals::GlobalWorldData>().unwrap();
    // Update the positions
    /*
    world
        .audio
        .update_ear_positions(global.camera_pos - global.camera_right, global.camera_pos + global.camera_right);
    */
}

// Create the audio system
pub fn system(world: &mut World) {
    world.ecs.systems.builder().with_run_event(run).build();
}
