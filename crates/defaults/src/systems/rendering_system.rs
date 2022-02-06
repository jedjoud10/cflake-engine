use main::{
    core::{Context, WriteContext},
    ecs::event::EventKey,
    rendering::{
        object::{ObjectID},
        pipeline::pipec,
    },
};

// The rendering system update loop
fn run(context: &mut Context, data: EventKey) {
    // For each renderer, we must update it's pipeline transform and other values
    let (mut query, _) = data.decompose().unwrap();
    let read = context.read().unwrap();
    let pipeline = read.pipeline.read();
    for (_, components) in query.lock().iter() {
        let renderer = components.get_component::<crate::components::Renderer>().unwrap();
        let transform = components.get_component::<crate::components::Transform>().unwrap();
        let renderer_object_id = renderer.object_id;
        if renderer_object_id.is_some() /* && components.component_update::<crate::components::Transform>().unwrap()  */{
            // Update the values if our renderer is valid
            let matrix = transform.calculate_matrix();
            pipec::update_callback(&pipeline, move |pipeline, _| {
                let gpu_renderer = pipeline.get_renderer_mut(renderer_object_id);
                if let Some(gpu_renderer) = gpu_renderer {
                    gpu_renderer.update_matrix(matrix);
                }
            }).unwrap();
        }
    }
}

// An event fired whenever we add multiple new renderer entities
fn added_entities(context: &mut Context, data: EventKey) {
    // For each renderer, we must create it's pipeline renderer construction task
    let (mut query, _) = data.decompose().unwrap();
    for (_, components) in query.lock().iter_mut() {
        // Get the pipeline first
        let read = context.read().unwrap();
        let pipeline = read.pipeline.read();

        // Get the CPU renderer that we must construct
        let mut renderer = components.get_component_mut::<crate::components::Renderer>().unwrap();
        let cpu_renderer = renderer.renderer.take().unwrap();
        let object_id = pipec::construct(&pipeline, cpu_renderer).unwrap();
        renderer.object_id = object_id;
    }
}

// An event fired whenever we remove multiple renderer entities
fn removed_entities(context: &mut Context, data: EventKey) {
    // For each renderer, we must dispose of it's GPU renderer
    let (mut query, _) = data.decompose().unwrap();
    for (_, components) in query.lock().iter_mut() {
        // Get the pipeline first
        let read = context.read().unwrap();
        let pipeline = read.pipeline.read();

        // Then get the ID of the GPU renderer
        let mut renderer = components.get_component_mut::<crate::components::Renderer>().unwrap();
        let id = renderer.object_id;
        renderer.object_id = ObjectID::default();

        // And create the task to dispose of it
        pipec::deconstruct(&pipeline, id);
    }
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
