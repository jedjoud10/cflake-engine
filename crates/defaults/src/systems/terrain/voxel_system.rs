use main::{ecs::{entity::EntityID, component::ComponentQuery}, rendering::pipeline::{Pipeline, pipec}, core::{WriteContext, Context}};

// Start generating the voxel data for a specific chunk
fn start_generation(terrain: &mut crate::globals::Terrain, pipeline: &Pipeline, chunk: &mut crate::components::Chunk, id: EntityID) {
}
// Finish generating the voxel data and read it back, then store it into the chunk
fn finish_generation(terrain: &mut crate::globals::Terrain, pipeline: &Pipeline, chunk: &mut crate::components::Chunk) {
}


// The voxel systems' update loop
fn run(context: &mut Context, query: ComponentQuery) {
    let mut write = context.write();
    // Get the pipeline without angering the borrow checker
    let pipeline_ = write.pipeline.clone();
    let pipeline = pipeline_.read();
    
    let terrain = write.ecs.global_mut::<crate::globals::Terrain>();
    if let Ok(mut terrain) = terrain {
        // For each chunk in the terrain, we must create it's respective voxel data, if possible
        if !terrain.generating {
            // We are not currently generating the voxel data, so we should start generating some for the first chunk that we come across that needs it
            query.update_all_breakable(|components| {
                // We break out at the first chunk if we start generating it's voxel data
                let mut chunk = components.component_mut::<crate::components::Chunk>().unwrap();
                if chunk.voxel_data.is_none() {
                    // We must start generating the voxel data for this chunk
                    start_generation(&mut *terrain, &pipeline, &mut *chunk, components.get_entity_id().unwrap());
                    None
                } else {
                    Some(())
                }
            })
        } else {
            // We must check if we have finished generating or not
            if pipec::did_tasks_execute(&[
                terrain.compute,
                terrain.compute_second,
                terrain.read_counters,
                terrain.read_final_voxels
            ], &pipeline) {
                // We will now update the chunk data to store our new voxel data
                let id = terrain.chunk_id.unwrap();
                query.update(id, |components| {
                    // Get our chunk and set it's new data
                    let mut chunk = components.component_mut::<crate::components::Chunk>().unwrap();
                    finish_generation(&mut *terrain, &*pipeline, &mut *chunk);
                });
            }
        }
    }
}
// Create a voxel system
pub fn system(write: &mut WriteContext) {
    write
        .ecs
        .create_system_builder()
        .set_run_event(run)
        .link::<crate::components::Transform>()
        .link::<crate::components::Chunk>()
        .build()
}
