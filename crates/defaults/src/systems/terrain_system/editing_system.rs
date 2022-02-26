use world::ecs::event::EventKey;
use world::World;
// A system that will handle terrain edits
fn run(world: &mut World, _data: EventKey) {
    // Camera values
    let global = world
        .globals
        .get_global::<crate::globals::GlobalWorldData>()
        .unwrap();
    let camera_position = global.camera_pos;
    let camera_forward = global.camera_forward;
    // Get the terrain global
    if let Ok(mut terrain) = world.globals.get_global_mut::<crate::globals::Terrain>() {
        // Editing manager
        let terrain = &mut *terrain;
        let chunks_to_regenerate = terrain
            .editing_manager
            .get_influenced_chunks(&terrain.chunks_manager.octree.lock().inner);
        if !chunks_to_regenerate.is_empty() {
            // Regenerate the specified chunks
            for coords in chunks_to_regenerate {
                terrain.regenerate_chunk(coords, camera_position, camera_forward);
            }
            // Also set the packed edits since we will need to update them on the GPU
            let packed = terrain.editing_manager.convert();
            terrain.voxel_generator.packed_edits_num = packed.len();
            terrain.voxel_generator.packed_edits_update = Some(packed);
        } else {
            terrain.voxel_generator.packed_edits_update = None;
        }
        terrain.chunks_manager.update_priorities();
    }
}

// Create the system
pub fn system(world: &mut World) {
    world.ecs.build_system().with_run_event(run).build();
}
