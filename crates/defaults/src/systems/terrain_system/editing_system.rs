use world::ecs::component::ComponentQuerySet;
use world::rendering::advanced::storages::Buffer;
use world::World;
// A system that will handle terrain edits
fn run(world: &mut World, _data: ComponentQuerySet) {
    // Get the terrain global
    if let Ok(terrain) = world.globals.get_mut::<crate::globals::Terrain>() {
        // Editing manager
        let terrain = &mut *terrain;
        let chunks_to_regenerate = terrain.editer.get_influenced_chunks(&terrain.manager.octree.inner);
        if !chunks_to_regenerate.is_empty() {
            // Regenerate the specified chunks
            for coords in chunks_to_regenerate {
                terrain.regenerate_chunk(coords);
            }
            // Also set the packed edits since we will need to update them on the GPU
            let packed = terrain.editer.convert();
            terrain.generator.ssbo_edits.storage_mut().write(&packed);
        }
        terrain.manager.update_priorities();
    }
}

// Create the system
pub fn system(world: &mut World) {
    world.ecs.systems.builder(&mut world.events.ecs).event(run).build().unwrap();
}
