use main::{
    core::{Context, WriteContext},
    ecs::component::ComponentQuery,
    rendering,
};

// The rendering system update loop
fn run(context: Context, query: ComponentQuery) {
    // For each renderer, we must update it's pipeline transform and other values
    let read = context.read();
    let pipeline = read.pipeline.read().unwrap();
    let _i = std::time::Instant::now();
    let tasks = query.update_all_map(move |components| {
        let renderer = components.component::<crate::components::Renderer>().unwrap();
        let transform = components.component::<crate::components::Transform>().unwrap();
        let renderer_object_id = &renderer.object_id;
        if renderer_object_id.valid() {
            // Update the values if our renderer is valid
            Some(rendering::object::PipelineTask::UpdateRendererMatrix(*renderer_object_id, transform.calculate_matrix()))
        } else {
            None
        }
    });

    // Since we have all the tasks, we can send them as a batch
    rendering::pipeline::pipec::task_batch(tasks, &*pipeline);
}

// An event fired whenever we add multiple new renderer entities
fn added_entities(context: Context, query: ComponentQuery) {
    let share = context.share();
    // For each renderer, we must create it's pipeline renderer construction task
    query.update_all(move |components| {
        // Get the pipeline first
        let read = share.read();
        let pipeline = read.pipeline.read().unwrap();

        // Get the CPU renderer that we must construct
        let mut renderer = components.component_mut::<crate::components::Renderer>().unwrap();
        let cpu_renderer = renderer.renderer.take().unwrap();
        let object_id = rendering::pipeline::pipec::construct(cpu_renderer, &*pipeline);
        renderer.object_id = object_id;
    })
}

// Create the rendering system
pub fn system(write: &mut WriteContext) {
    write
        .ecs
        .create_system_builder()
        .set_run_event(run)
        .set_added_entities_event(added_entities)
        .link::<crate::components::Renderer>()
        .link::<crate::components::Transform>()
        .build();
}
