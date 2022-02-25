use world::World;
use world::ecs::event::EventKey;

// Update the position of the left and right ears
fn run(world: &mut World, _data: EventKey) {
    // Global
    let global = world
        .globals
        .get_global::<crate::globals::GlobalWorldData>()
        .unwrap();
    // Update the positions
    world.audio.update_ear_positions(
        global.camera_pos - global.camera_right,
        global.camera_pos + global.camera_right,
    );
}

// Create the audio system
pub fn system(world: &mut World) {
    world.ecs.build_system().with_run_event(run).build();
}
