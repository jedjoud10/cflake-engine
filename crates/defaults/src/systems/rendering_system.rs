use world::{
    ecs::{
        component::{ComponentKey, RefComponentFetcher},
        event::EventKey,
    },
    rendering::pipeline::{RenderedModel, RenderingCamera, RenderingSettings, ShadowedModel},
    World,
};

use crate::components::{Camera, Light, Renderer, Transform};

// Get the camera transform and camera data
fn get_camera(world: &World) -> Option<(RefComponentFetcher, ComponentKey, ComponentKey)> {
    // Get the entity
    let global = world.globals.get::<crate::globals::GlobalWorldData>().unwrap();
    world
        .ecs
        .entities
        .get(global.main_camera)
        .map(|camera_entity| {
            // And fetch it's linked component keys
            let camera = camera_entity.get_linked::<Camera>().unwrap();
            let transform = camera_entity.get_linked::<Transform>().unwrap();

            // Then, we can fetch the actual components
            let fetcher = RefComponentFetcher::new(&world.ecs.components);
            (fetcher, camera, transform)
        })
        .ok()
}

// The rendering system update loop
fn run(world: &mut World, mut data: EventKey) {
    // Render the world
    let query = data.as_query_mut().unwrap();

    // Before we do anything, we must update each model matrix if it needs to be updated
    for (_, components) in query.iter_mut() {
        // Only update if we need to
        if components.was_mutated::<Transform>().unwrap_or_default() || components.was_mutated::<Renderer>().unwrap_or_default() {
            let transform = components.get::<Transform>().unwrap();
            let matrix = transform.transform_matrix();
            let renderer = components.get_mut::<Renderer>().unwrap();
            renderer.matrix = matrix;
        }
    }

    // Turn the component query into a list of RenderedModels and ShadowedModels
    let mut models: Vec<RenderedModel> = Vec::with_capacity(query.len());
    let mut shadowed: Vec<ShadowedModel> = Vec::with_capacity(query.len());

    // Add the normal models
    for (_, components) in query.iter() {
        // We do a bit of borrowing
        let renderer = components.get::<Renderer>().unwrap();
        models.push(RenderedModel {
            mesh: &renderer.mesh,
            matrix: &renderer.matrix,
            material: &renderer.material,
        });
    }

    // Add the shadowed models
    for (_, components) in query.iter() {
        // We do a bit of borrowing
        let renderer = components.get::<Renderer>().unwrap();
        // Only if this is shadowed
        if renderer.shadowed {
            shadowed.push(ShadowedModel {
                mesh: &renderer.mesh,
                matrix: &renderer.matrix,
            });
        }
    }

    // Fetch the camera component
    let camera_data = get_camera(world);
    if let Some((fetcher, camera, transform)) = camera_data {
        let camera = fetcher.get::<Camera>(camera).unwrap();
        let transform = fetcher.get::<Transform>(transform).unwrap();

        // Camera settings
        let camera = RenderingCamera {
            position: transform.position,
            rotation: transform.rotation,
            viewm: camera.viewm,
            projm: camera.projm,
            clip_planes: camera.clip_planes,

            // Math moment
            projm_viewm: camera.projm * camera.viewm,
        };
        world.pipeline.camera = camera;

        // Get all the lights that are in the world
        let fetcher = RefComponentFetcher::new(&world.ecs.components);
        let lights = fetcher.get_all::<Light>().unwrap();

        // Rendering settings
        let settings = RenderingSettings {
            normal: models.as_slice(),
            shadowed: shadowed.as_slice(),
            lights: lights,
        };

        // Render
        let renderer = &world.renderer;
        let pipeline = &world.pipeline;
        renderer.render(pipeline, settings);
    }
}

// An event fired whenever we add multiple new renderer entities
fn added_entities(_world: &mut World, _data: EventKey) {}

// An event fired whenever we remove multiple renderer entities
fn removed_entities(_world: &mut World, _data: EventKey) {}

// Create the rendering system
pub fn system(world: &mut World) {
    world
        .ecs
        .systems
        .builder()
        .with_run_event(run)
        .with_added_entities_event(added_entities)
        .with_removed_entities_event(removed_entities)
        .link::<Renderer>()
        .link::<Transform>()
        .build();
}
