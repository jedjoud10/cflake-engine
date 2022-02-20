use main::{
    core::World,
    ecs::{
        event::EventKey,
        rayon::iter::{IntoParallelRefIterator, ParallelIterator},
    },
    rendering::{object::ObjectID, pipeline::pipec},
};

// The rendering system update loop
fn run(world: &mut World, mut data: EventKey) {
    // For each renderer, we must update it's pipeline transform and other values
    let query = data.as_query_mut().unwrap();
    let pipeline = world.pipeline.read();
    // Le local struct
    struct RendererUpdatedMatrixUnit {
        renderer_id: ObjectID<main::rendering::basics::renderer::Renderer>,
        matrix: veclib::Matrix4x4<f32>,
    }

    let result = query
        .write()
        .par_iter()
        .filter_map(|(_, components)| {
            let renderer = components.get_component::<crate::components::Renderer>().unwrap();
            let transform = components.get_component::<crate::components::Transform>().unwrap();
            let renderer_id = renderer.object_id;
            // Only update if we have a valid renderer and if we changed our transform
            if renderer_id.is_some() && components.was_mutated::<crate::components::Transform>().unwrap_or_default() {
                // Update the values if our renderer is valid
                let matrix = transform.calculate_matrix();
                Some(RendererUpdatedMatrixUnit { renderer_id, matrix })
            } else {
                None
            }
        })
        .collect::<Vec<RendererUpdatedMatrixUnit>>();

    // Now we can send ALL of the new update matrices
    pipec::update_callback(&pipeline, move |pipeline, _| {
        for x in result {
            let gpu_renderer = pipeline.renderers.get_mut(x.renderer_id);
            if let Some(gpu_renderer) = gpu_renderer {
                gpu_renderer.matrix = x.matrix;
            }
        }
    });

    // Also update the direction of the sun (internally stored as a Directional Light)
    let global = world.globals.get_global::<crate::globals::GlobalWorldData>().unwrap();
    let (quat, id) = (global.sun_quat, pipeline.defaults.as_ref().unwrap().sun);
    pipec::update_callback(&pipeline, move |pipeline, _| {
        // Update the sun's light source, if possible
        if let Some(light) = pipeline.light_sources.get_mut(id) {
            *light._type.as_directional_mut().unwrap() = quat;
        }
    });
}

// An event fired whenever we add multiple new renderer entities
fn added_entities(world: &mut World, mut data: EventKey) {
    // For each renderer, we must create it's pipeline renderer construction task
    let query = data.as_query_mut().unwrap();
    for (_, components) in query.write().iter_mut() {
        // Get the pipeline first
        let pipeline = world.pipeline.read();

        // Get the CPU renderer that we must construct
        let matrix = components.get_component::<crate::components::Transform>().unwrap().calculate_matrix();
        let mut renderer = components.get_component_mut::<crate::components::Renderer>().unwrap();
        let mut cpu_renderer = renderer.renderer.take().unwrap();
        cpu_renderer.matrix = matrix;
        let object_id = pipec::construct(&pipeline, cpu_renderer).unwrap();
        renderer.object_id = object_id;
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
        let mut renderer = components.get_component_mut::<crate::components::Renderer>().unwrap();
        let id = renderer.object_id;
        renderer.object_id = ObjectID::default();

        // And create the task to dispose of it
        pipec::deconstruct(&pipeline, id);
    }
}

// Create the rendering system
pub fn system(world: &mut World) {
    world
        .ecs
        .create_system_builder()
        .with_run_event(run)
        .with_added_entities_event(added_entities)
        .with_removed_entities_event(removed_entities)
        .link::<crate::components::Renderer>()
        .link::<crate::components::Transform>()
        .build();
}
