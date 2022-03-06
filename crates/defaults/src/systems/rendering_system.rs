use world::{
    ecs::event::EventKey,
    rendering::{
        basics::uniforms::StoredUniforms,
        pipeline::{RenderedModel, RenderingCamera, RenderingSettings, ShadowedModel},
    },
    World,
};

// The rendering system update loop
fn run(world: &mut World, mut data: EventKey) {
    // Render the world
    let renderer = &mut world.renderer;
    let pipeline = &mut world.pipeline;
    let query = data.as_query_mut().unwrap();

    // Before we do anything, we must update each model matrix if it needs to be updated
    for (_, components) in query.iter_mut() {
        // Only update if we need to
        if components.was_mutated::<crate::components::Transform>().unwrap_or_default() {
            let transform = components.get::<crate::components::Transform>().unwrap();
            let matrix = transform.transform_matrix();
            let renderer = components.get_mut::<crate::components::Renderer>().unwrap();
            renderer.matrix = matrix;
        }
    }

    // Turn the component query into a list of RenderedModels and ShadowedModels
    let mut models: Vec<RenderedModel> = Vec::with_capacity(query.len());
    let mut shadowed: Vec<ShadowedModel> = Vec::with_capacity(query.len());

    // Add the normal models
    for (_, components) in query.iter() {
        // We do a bit of borrowing
        let renderer = components.get::<crate::components::Renderer>().unwrap();
        models.push(RenderedModel {
            mesh: &renderer.mesh,
            matrix: &renderer.matrix,
            material: &renderer.material,
            uniforms: &renderer.uniforms,
        });
    }

    // Add the shadowed models
    for (_, components) in query.iter() {
        // We do a bit of borrowing
        let renderer = components.get::<crate::components::Renderer>().unwrap();
        // Only if this is shadowed
        if renderer.shadowed {
            shadowed.push(ShadowedModel {
                mesh: &renderer.mesh,
                matrix: &renderer.matrix,
            });
        }
    }

    // Camera settings
    let camera = RenderingCamera {
        position: todo!(),
        rotation: todo!(),
        viewm: todo!(),
        projm: todo!(),
        projm_viewm: todo!(),
        forward: todo!(),
        clip_planes: todo!(),
    };

    // Rendering settings
    let settings = RenderingSettings {
        normal: models.as_slice(),
        shadowed: shadowed.as_slice(),
        camera: camera,
    };

    // Render
    renderer.render(pipeline, settings);
}

// An event fired whenever we add multiple new renderer entities
fn added_entities(world: &mut World, mut data: EventKey) {}

// An event fired whenever we remove multiple renderer entities
fn removed_entities(world: &mut World, mut data: EventKey) {}

// Create the rendering system
pub fn system(world: &mut World) {
    world
        .ecs
        .systems
        .builder()
        .with_run_event(run)
        .with_added_entities_event(added_entities)
        .with_removed_entities_event(removed_entities)
        .link::<crate::components::Renderer>()
        .link::<crate::components::Transform>()
        .build();
}
