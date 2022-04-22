use crate::components::{Camera, Light, Renderer, RendererFlags, Transform};
use world::{
    ecs::{added, always, modified, never, or},
    rendering::{
        basics::lights::LightTransform,
        pipeline::{RenderedModel, RenderingCamera, RenderingSettings, ShadowedModel},
    },
    World,
};

// The 3D scene renderer
fn run(world: &mut World) {
    let global = world.globals.get::<crate::globals::GlobalWorldData>().unwrap();
    // Get the camera if possible, and stop rendering if we are unable to
    let entry = world.ecs.entry(global.camera);
    // If we have the camera, update it in the pipeline
    // If we do not, stop rendering
    if let Some(mut entry) = entry {
        // Get the components and build a RenderingCamera out of them
        let (position, rotation, forward, up) = {
            let transform = entry.get::<Transform>().unwrap();
            (transform.position, transform.rotation, transform.forward(), transform.up())
        };

        // Update the camera's view matrix if needed
        let update = entry.was_mutated::<Transform>().unwrap();
        let camera = entry.get_mut::<Camera>().unwrap();
        camera.update_projection_matrix(world.pipeline.window().dimensions().w as f32, world.pipeline.window().dimensions().h as f32);
        if update {
            camera.update_view_matrix(position, forward, up);
        }
        // Rendering camera settings
        let camera = RenderingCamera {
            position: position,
            rotation: rotation,
            viewm: camera.viewm,
            projm: camera.projm,
            clip_planes: camera.clip_planes,

            // Math moment
            projm_viewm: camera.projm * camera.viewm,
        };
        *world.pipeline.camera_mut() = camera;
    } else {
        // There isn't a camera, no need to render anything
        // Force a clear of the default framebuffer, since we won't be overwritting it
        world.renderer.default_mut().clear();
        return;
    }

    // A bit of trolling yea?
    let query = world.ecs.query::<(&mut Transform, &Light)>();
    for (transform, _) in query {
        transform.rotation.rotate_x(0.09 * world.time.delta());
        //transform.rotation.rotate_y(0.07 * world.time.delta());
    }

    // Update the matrices of renderers, only if the transforms os said renderers were externally modified
    let filter = or(modified::<Renderer>(), added::<Renderer>());
    let query = world.ecs.query_with::<(&Transform, &mut Renderer), _>(filter);
    for (transform, renderer) in query {
        // Update the matrix if we need to
        renderer.matrix = transform.transform_matrix();
        renderer.flags.insert(RendererFlags::MATRIX_UPDATE);
    }
    // Get all the visible objects in the world, first of all
    let query = world.ecs.try_view::<&Renderer>().unwrap();
    let models: Vec<RenderedModel> = Vec::from_iter(query.filter_map(|renderer| {
        // No need to render an invisible entity
        if renderer.flags.contains(RendererFlags::VISIBLE) {
            Some(RenderedModel {
                mesh: &renderer.mesh,
                matrix: &renderer.matrix,
                material: &renderer.material,
            })
        } else {
            None
        }
    }));

    // Next, get all the shadowed models (used for shadow-mapping)
    let query = world.ecs.try_view::<&Renderer>().unwrap();
    let mut redraw_shadows = false;
    let shadowed: Vec<ShadowedModel> = Vec::from_iter(query.filter_map(|renderer| {
        // No need to draw shadows for invisible renderers
        let f = renderer.flags;
        if f.contains(RendererFlags::SHADOW_CASTER) && f.contains(RendererFlags::VISIBLE) {
            // We must only redraw shadows if at least one of the shadow caster objects needs to be redrawn
            redraw_shadows |= f.contains(RendererFlags::MATRIX_UPDATE);
            Some(ShadowedModel {
                mesh: &renderer.mesh,
                matrix: &renderer.matrix,
            })
        } else {
            None
        }
    }));

    // Get all the lights that are in the scene
    let query = world.ecs.try_view::<(&Transform, &Light)>().unwrap();
    let lights = query
        .map(|(transform, light)| {
            // Convert into rendering structs
            let _type = &light.0;
            let transform = LightTransform {
                position: &transform.position,
                rotation: &transform.rotation,
            };

            // And pack into a tuple
            (_type, transform)
        })
        .collect::<Vec<_>>();

    // Detect if we need to redraw shadows because of the light source updating
    redraw_shadows |= world.ecs.try_view_with::<(&Transform, &Light), _>(modified::<Transform>()).unwrap().count() > 0;

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
    world.events.insert(run);
}
