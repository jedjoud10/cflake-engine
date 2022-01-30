use main::{
    core::{Context, WriteContext},
    ecs::component::ComponentQuery,
    rendering::{self, object::PipelineTask},
};

// The rendering system update loop
fn run(context: &mut Context, query: ComponentQuery) {
    // For each renderer, we must update it's pipeline transform and other values
    let read = context.read();
    let pipeline = read.pipeline.read();
    let _i = std::time::Instant::now();
    let storage = main::threads::SharedVec::<Option<PipelineTask>>::new(query.count());
    query.update_all_threaded(|execution_id, components| {
        let renderer = components.component::<crate::components::Renderer>().unwrap();
        let transform = components.component::<crate::components::Transform>().unwrap();
        let renderer_object_id = &renderer.object_id;
        let task = if renderer_object_id.valid() {
            // Update the values if our renderer is valid
            Some(rendering::object::PipelineTask::UpdateRendererMatrix(*renderer_object_id, transform.calculate_matrix()))
        } else {
            None
        };
        // Write the task
        let option = storage.write(execution_id).unwrap();
        *option = task;
    });

    // Since we have all the tasks, we can send them as a batch
    let vec = storage.into_inner().into_iter().filter_map(|x| x).collect::<Vec<_>>();
    rendering::pipeline::pipec::task_batch(vec, &*pipeline);
}

// An event fired whenever we add multiple new renderer entities
fn added_entities(context: &mut Context, query: ComponentQuery) {
    // For each renderer, we must create it's pipeline renderer construction task
    query.update_all(move |components| {
        // Get the pipeline first
        let read = context.read();
        let pipeline = read.pipeline.read();

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
