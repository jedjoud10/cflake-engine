use main::{
    core::{Context, WriteContext},
    ecs::{component::ComponentQuery, self, entity::EntityID},
    terrain::{DEFAULT_LOD_FACTOR, ChunkCoords},
};

// The voxel systems' update loop
fn run(mut context: Context, query: ComponentQuery) {
    let mut write = context.write();
    let terrain = write.ecs.global_mut::<crate::globals::Terrain>();
    if let Ok(mut terrain) = terrain {
        let pipeline = write.pipeline.read().unwrap();
        if terrain.generating.is_none() {
            let es = main::rendering::advanced::compute::ComputeShaderExecutionSettings::new(terrain.compute_shader, (32, 32, 32));
            let partial = main::rendering::pipeline::pipec::tracked_task(main::rendering::object::PipelineTrackedTask::RunComputeShader(terrain.compute_shader, es), None, &*pipeline); 
            let full = main::rendering::pipeline::pipec::tracked_finalizer(vec![partial], &*pipeline).unwrap();
            terrain.generating = Some(full);
            println!("Started on {}", write.time.frame_count);
        } else {
            if main::rendering::pipeline::pipec::has_task_executed(terrain.generating.unwrap(), &*pipeline).unwrap() {
                println!("Finished on {}", write.time.frame_count);
            }
        }
    }
    /*
    // Get the first pending chunk and generate it's voxel data, only if we have time though
    query.update_all_breakable(|components| {
        // This chunk will become a pending chunk

    })
    */
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
