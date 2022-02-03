use main::{
    core::{Context, WriteContext},
    ecs::component::ComponentQuery,
    rendering::{
        self,
        object::{ObjectID, PipelineTask},
    },
};

// The rendering system update loop
fn run(context: &mut Context, query: ComponentQuery) {
    // For each renderer, we must update it's pipeline transform and other values
    let read = context.read().unwrap();
    let pipeline = read.pipeline.read();
    let _i = std::time::Instant::now();
    let storage = main::threads::SharedVec::<Option<PipelineTask>>::new(query.get_entity_count());
    query.update_all_threaded(|execution_id, components| {
        let renderer = components.get_component::<crate::components::Renderer>().unwrap();
        let transform = components.get_component::<crate::components::Transform>().unwrap();
        let renderer_object_id = &renderer.object_id;
        let task = if renderer_object_id.is_some() {
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
    let vec = storage.into_inner().into_iter().flatten().collect::<Vec<_>>();
    rendering::pipeline::pipec::task_batch(vec, &pipeline);
}

// An event fired whenever we add multiple new renderer entities
fn added_entities(context: &mut Context, query: ComponentQuery) {
    // For each renderer, we must create it's pipeline renderer construction task
    query.update_all(move |components| {
        // Get the pipeline first
        let read = context.read().unwrap();
        let pipeline = read.pipeline.read();

        // Get the CPU renderer that we must construct
        let mut renderer = components.get_component_mut::<crate::components::Renderer>().unwrap();
        let cpu_renderer = renderer.renderer.take().unwrap();
        let object_id = rendering::pipeline::pipec::construct(cpu_renderer, &pipeline);
        renderer.object_id = object_id;
    })
}

// An event fired whenever we remove multiple renderer entities
fn removed_entities(context: &mut Context, query: ComponentQuery) {
    // For each renderer, we must dispose of it's GPU renderer
    query.update_all(move |components| {
        // Get the pipeline first
        let read = context.read().unwrap();
        let pipeline = read.pipeline.read();

        // Then get the ID of the GPU renderer
        let mut renderer = components.get_component_mut::<crate::components::Renderer>().unwrap();
        let id = renderer.object_id;
        renderer.object_id = ObjectID::default();

        // And create the task to dispose of it
        rendering::pipeline::pipec::task(PipelineTask::DisposeRenderer(id), &pipeline);
    })
}

// Create the rendering system
pub fn system(write: &mut WriteContext) {
    write
        .ecs
        .create_system_builder()
        .with_run_event(run)
        .with_added_entities_event(added_entities)
        .with_removed_entities_event(removed_entities)
        .link::<crate::components::Renderer>()
        .link::<crate::components::Transform>()
        .build();
}
