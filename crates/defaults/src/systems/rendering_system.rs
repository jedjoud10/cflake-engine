use world::{
    ecs::{
        event::EventKey,
        rayon::iter::{IntoParallelRefIterator, ParallelIterator},
    },
    rendering::{object::ObjectID, pipeline::pipec},
    World,
};

// The rendering system update loop
fn run(world: &mut World, mut data: EventKey) {
    // For each renderer, we must update it's pipeline transform and other values
    let query = data.as_query_mut().unwrap();
    let pipeline = world.pipeline.read();
    // Le local struct
    struct RendererUpdatedMatrixUnit {
        renderer_id: ObjectID<world::rendering::basics::renderer::Renderer>,
        matrix: veclib::Matrix4x4<f32>,
    }

    let result = query
        .write()
        .par_iter()
        .filter_map(|(_, components)| {
            let renderer = components
                .get::<crate::components::Renderer>()
                .unwrap();
            let transform = components
                .get::<crate::components::Transform>()
                .unwrap();
            let renderer_id = renderer.id;
            // Only update if we have a valid renderer and if we changed our transform
            if renderer_id.is_some()
                && components
                    .was_mutated::<crate::components::Transform>()
                    .unwrap_or_default()
            {
                // Update the values if our renderer is valid
                let matrix = transform.calculate_matrix();
                Some(RendererUpdatedMatrixUnit {
                    renderer_id,
                    matrix,
                })
            } else {
                None
            }
        })
        .collect::<Vec<RendererUpdatedMatrixUnit>>();

    // Now we can send ALL of the new update matrices
    if !result.is_empty() {
        pipec::update_callback(&pipeline, move |pipeline, _| {
            for x in result {
                let gpu_renderer = pipeline.renderers.get_mut(x.renderer_id);
                if let Some(gpu_renderer) = gpu_renderer {
                    gpu_renderer.matrix = x.matrix;
                }
            }
        });
    }
}

// An event fired whenever we add multiple new renderer entities
fn added_entities(world: &mut World, mut data: EventKey) {
    // For each renderer, we must create it's pipeline renderer construction task
    let query = data.as_query_mut().unwrap();
    for (_, components) in query.write().iter_mut() {
        // Get the pipeline first
        let pipeline = world.pipeline.read();

        // Get the CPU renderer that we must construct
        let matrix = components
            .get::<crate::components::Transform>()
            .unwrap()
            .calculate_matrix();
        let mut renderer = components
            .get_mut::<crate::components::Renderer>()
            .unwrap();
        let mut cpu_renderer = renderer.inner.take().unwrap();
        cpu_renderer.matrix = matrix;
        let object_id = pipec::construct(&pipeline, cpu_renderer).unwrap();
        renderer.id = object_id;
    }
}

// An event fired whenever we remove multiple renderer entities
fn removed_entities(world: &mut World, mut data: EventKey) {
    // For each renderer, we must dispose of it's GPU renderer
    let query = data.as_query_mut().unwrap();
    for (_, components) in query.write().iter_mut() {
        // Get the pipeline first
        let pipeline = world.pipeline.read();

        // Then get the ID of the GPU renderer
        let mut renderer = components
            .get_mut::<crate::components::Renderer>()
            .unwrap();
        let id = renderer.id;
        renderer.id = ObjectID::default();

        // And create the task to dispose of it
        pipec::deconstruct(&pipeline, id);
    }
}

// Create the rendering system
pub fn system(world: &mut World) {
    world
        .ecs
        .build_system()
        .with_run_event(run)
        .with_added_entities_event(added_entities)
        .with_removed_entities_event(removed_entities)
        .link::<crate::components::Renderer>()
        .link::<crate::components::Transform>()
        .build();
}
