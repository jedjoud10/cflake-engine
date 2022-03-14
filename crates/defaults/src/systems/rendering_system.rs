use crate::components::{Camera, Light, Renderer, Transform};
use world::{
    ecs::component::{ComponentQueryParameters, ComponentQuerySet},
    rendering::{
        basics::lights::LightTransform,
        pipeline::{RenderedModel, RenderingCamera, RenderingSettings, ShadowedModel},
    },
    World,
};

// The rendering system update loop
fn run(world: &mut World, mut data: ComponentQuerySet) {
    // Get the camera if possible
    let global = world.globals.get::<crate::globals::GlobalWorldData>().unwrap();
    let camquery = data.get_mut(1).unwrap();
    let linked = camquery.all.get(&global.main_camera);
    let camera = linked.map(|linked| {
        let camera = linked.get::<Camera>().unwrap();
        let transform = linked.get::<Transform>().unwrap();
        // Camera settings
        RenderingCamera {
            position: transform.position,
            rotation: transform.rotation,
            viewm: camera.viewm,
            projm: camera.projm,
            clip_planes: camera.clip_planes,

            // Math moment
            projm_viewm: camera.projm * camera.viewm,
        }
    });

    // If there isn't a camera, no need to render anything
    if camera.is_none() {
        return;
    }
    *world.pipeline.camera_mut() = camera.unwrap();

    // Render the world
    let query = data.get_mut(0).unwrap();

    // Before we do anything, we must update each model matrix if it needs to be updated
    for (_, components) in query.all.iter_mut() {
        // Only update if we need to
        if components.was_mutated::<Transform>().unwrap_or_default() || components.was_mutated::<Renderer>().unwrap_or_default() {
            let transform = components.get::<Transform>().unwrap();
            let matrix = transform.transform_matrix();
            let renderer = components.get_mut::<Renderer>().unwrap();
            renderer.matrix = matrix;
        }
    }

    // Turn the component query into a list of RenderedModels and ShadowedModels
    let mut models: Vec<RenderedModel> = Vec::with_capacity(query.all.len());
    let mut shadowed: Vec<ShadowedModel> = Vec::with_capacity(query.all.len());

    // Add the normal models
    let query = data.get(0).unwrap();
    for (_, components) in query.all.iter() {
        // We do a bit of borrowing
        let renderer = components.get::<Renderer>().unwrap();
        models.push(RenderedModel {
            mesh: &renderer.mesh,
            matrix: &renderer.matrix,
            material: &renderer.material,
        });
    }

    // Add the shadowed models
    for (_, components) in query.all.iter() {
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

    // Get the lights
    let query = data.get(2).unwrap();
    let lights = query
        .all
        .iter()
        .map(|(_, linked)| {
            // Get the linked components
            let light = linked.get::<Light>().unwrap();
            let transform = linked.get::<Transform>().unwrap();

            // Convert
            (
                &light.light,
                LightTransform {
                    position: &transform.position,
                    rotation: &transform.rotation,
                },
            )
        })
        .collect::<Vec<_>>();

    // Rendering settings
    let settings = RenderingSettings {
        normal: models.as_slice(),
        shadowed: shadowed.as_slice(),
        lights: &lights,
    };

    // Render
    let renderer = &mut world.renderer;
    let pipeline = &world.pipeline;
    renderer.render(pipeline, settings);
}

// Create the rendering system
pub fn system(world: &mut World) {
    world
        .ecs
        .systems
        .builder()
        .event(run)
        .query(ComponentQueryParameters::default().link::<Renderer>().link::<Transform>())
        .query(ComponentQueryParameters::default().link::<Camera>().link::<Transform>())
        .query(ComponentQueryParameters::default().link::<Light>().link::<Transform>())
        .build();
}
