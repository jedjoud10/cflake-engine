use world::ecs::{added, or, EcsManager};
use world::math::shapes::ShapeType;
use world::rendering::advanced::storages::Buffer;
use world::terrain::editing::Edit;
use world::World;

use crate::components::Transform;
use crate::resources;
// A system that will handle terrain edits
fn run(world: &mut World) {
    // Get the terrain global
    if let Some(terrain) = world.resources.get_mut::<resources::Terrain>() {
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
    world.events.insert(run)
}
