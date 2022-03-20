use crate::components::{Camera, Light, Renderer, RendererFlags, Transform};
use world::{
    ecs::component::{ComponentQueryParams, ComponentQuerySet},
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
    let linked = camquery.all.get_mut(&global.main_camera);
    let camera = linked.map(|components| {
        // Get the linked components
        let (position, forward, up, rotation, mutated) = {
            let transform = components.get::<Transform>().unwrap();
            (
                transform.position,
                transform.forward(),
                transform.up(),
                transform.rotation,
                components.was_mutated::<Transform>().unwrap(),
            )
        };
        let camera = components.get_mut::<Camera>().unwrap();
        // And don't forget to update the camera matrices
        camera.update_projection_matrix(world.pipeline.window().dimensions().w as f32, world.pipeline.window().dimensions().h as f32);
        if mutated {
            camera.update_view_matrix(position, forward, up);
        }
        // Camera settings
        RenderingCamera {
            position,
            rotation,
            viewm: camera.viewm,
            projm: camera.projm,
            clip_planes: camera.clip_planes,

            // Math moment
            projm_viewm: camera.projm * camera.viewm,
        }
    });

    // If there isn't a camera, no need to render anything
    if camera.is_none() {
        // Force a clear of the default framebuffer, since we won't be overwritting it
        world.renderer.default_mut().clear();
        return;
    }
    *world.pipeline.camera_mut() = camera.unwrap();

    // Render the world
    let query = data.get_mut(0).unwrap();

    // Keep track if we need to redraw shadows
    let mut redraw_shadows = false;

    // Before we do anything, we must update each model matrix if it needs to be updated
    for (_, components) in query.all.iter_mut() {
        // Only update if we need to
        if components.was_mutated::<Transform>().unwrap() || components.was_mutated::<Renderer>().unwrap() {
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
        if renderer.flags.contains(RendererFlags::VISIBLE) {
            models.push(RenderedModel {
                mesh: &renderer.mesh,
                matrix: &renderer.matrix,
                material: &renderer.material,
            });
        }
    }

    // Add the shadowed models
    for (_, components) in query.all.iter() {
        // We do a bit of borrowing
        let renderer = components.get::<Renderer>().unwrap();
        // Only if this is shadowed
        if renderer.flags.contains(RendererFlags::SHADOWED) && renderer.flags.contains(RendererFlags::VISIBLE) {
            shadowed.push(ShadowedModel {
                mesh: &renderer.mesh,
                matrix: &renderer.matrix,
            });
            // Only redraw if we need to
            if components.was_mutated::<Renderer>().unwrap() || components.was_mutated::<Transform>().unwrap() {
                redraw_shadows = true;
            }
        }
    }

    // More shadow checks, just to be sure
    for (_, components) in query.delta.removed.iter().chain(query.delta.added.iter()) {
        // We do a bit of borrowing
        let renderer = components.get::<Renderer>().unwrap();
        // Only if this is shadowed
        if renderer.flags.contains(RendererFlags::SHADOWED) && renderer.flags.contains(RendererFlags::VISIBLE) {
            redraw_shadows = true;
            break;
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
        redraw_shadows,
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
        .builder(&mut world.events.ecs)
        .event(run)
        .query(ComponentQueryParams::default().link::<Renderer>().link::<Transform>())
        .query(ComponentQueryParams::default().link::<Camera>().link::<Transform>())
        .query(ComponentQueryParams::default().link::<Light>().link::<Transform>())
        .build()
        .unwrap();
}
