use main::core::World;
use main::ecs::event::EventKey;
// A system that will handle terrain edits
fn run(world: &mut World, _data: EventKey) {
    // Get the terrain global
    if let Ok(mut terrain) = world.globals.get_global_mut::<crate::globals::Terrain>() {
        // Editing manager
        let terrain = &mut *terrain;
        let chunks_to_regenerate = terrain.editing_manager.get_influenced_chunks(&terrain.chunks_manager.octree.inner);
        if !chunks_to_regenerate.is_empty() {
            // Regenerate the specified chunks
            for coords in chunks_to_regenerate {
                terrain.regenerate_chunk(coords);
            }
            // Also set the packed edits since we will need to update them on the GPU
            terrain.voxel_generator.packed_edits_update = Some(terrain.editing_manager.convert());
        } else {
            terrain.voxel_generator.packed_edits_update = None;
        }
    }
}

// Create the system
pub fn system(world: &mut World) {
    world.ecs.create_system_builder().with_run_event(run).build();
}
