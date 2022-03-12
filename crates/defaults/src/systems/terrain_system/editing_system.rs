use world::ecs::component::ComponentQuerySet;
use world::rendering::advanced::raw::Buffer;
use world::World;
// A system that will handle terrain edits
fn run(world: &mut World, _data: ComponentQuerySet) {
    // Get the terrain global
    if let Ok(terrain) = world.globals.get_mut::<crate::globals::Terrain>() {
        // Editing manager
        let terrain = &mut *terrain;
        let chunks_to_regenerate = terrain.editing_manager.get_influenced_chunks(&terrain.chunks_manager.octree.inner);
        if !chunks_to_regenerate.is_empty() {
            // Regenerate the specified chunks
            for coords in chunks_to_regenerate {
                terrain.regenerate_chunk(coords);
            }
            // Also set the packed edits since we will need to update them on the GPU
            let packed = terrain.editing_manager.convert();
            terrain.voxel_generator.ssbo_edits.storage_mut().write(&packed);
        }
        terrain.chunks_manager.update_priorities();
    }
}

// Create the system
pub fn system(world: &mut World) {
    world.ecs.systems.builder().event(run).build();
}
