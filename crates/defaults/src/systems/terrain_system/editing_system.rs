use main::core::World;
use main::ecs::event::EventKey;

// A system that will handle terrain edits
fn run(world: &mut World, _data: EventKey) {
    // Get the terrain global
    if let Ok(terrain) = world.globals.get_global_mut::<crate::globals::Terrain>() {
        // Editing manager
        let res = terrain.editing_manager.get_influenced_chunks(&terrain.chunks_manager.octree.inner);
    }
}

// Create the system
pub fn system(world: &mut World) {
    world
        .ecs
        .create_system_builder()
        .with_run_event(run)
        .build();
}
