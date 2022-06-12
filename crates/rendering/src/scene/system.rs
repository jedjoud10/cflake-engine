use crate::{context::Graphics, mesh::SubMesh, shader::Shader, material::{AlbedoMap, NormalMap, MaskMap}, prelude::{Texture2D, RGBA, Ranged, Texture, TextureMode, Sampling, Filter, Wrap}};
use ecs::EcsManager;
use world::{resources::Storage, World};
use super::SceneSettings;

// Initialization system that will setup the default textures and objects
pub fn init(world: &mut World) {
    // Get the storages and graphical context
    let (graphics, rgba, normal_maps, mask_maps) = world.get_mut::<(&mut Graphics, &mut Storage<AlbedoMap>, &mut Storage<NormalMap>, &mut Storage<MaskMap>)>().unwrap();
    let Graphics(device, ctx) = graphics;

    // RGBA 8 bits per channel texture
    type Tex = Texture2D<RGBA<Ranged<u8>>>;
    let sampling = Sampling::new(Filter::Nearest, Wrap::Repeat);

    // Create the default black texture
    let black = Texture2D::new(ctx, TextureMode::Dynamic, vek::Extent2::one(), sampling, false, &[RGBA])

    // Create the default white texture
}


// Update system that will execute each frame to try to render the scene
pub fn rendering(world: &mut World) {
    // Get the graphics context, ecs, and the main scene renderer
    let (graphics, ecs, settings) = world.get_mut::<(&mut Graphics, &mut EcsManager, &SceneSettings)>().unwrap();
    let Graphics(device, context) = graphics;

    // Can we render the scene?
    if !settings.can_render() {
        return;
    }
    let settings = settings.clone();

    // Update all the renderer components
    let renderers = context.extract_material_renderer();
    
    // Render all the material surfaces
    let stats = renderers.into_iter().map(|elem| elem.render(world, &settings)).collect::<Vec<_>>();
}

// Main camera system that will update the camera matrices

/*
// Recalculate the AABB of a given renderer using a 4x4 translation and rotation matrix (model matrix)
// TODO: Use the code from https://stackoverflow.com/questions/6053522/how-to-recalculate-axis-aligned-bounding-box-after-translate-rotate
// For optimization reasons
fn project_aabb(aabb: &AABB, m: vek::Mat4<f32>) -> AABB {
    // Keep track of the min/max positions
    let mut max = vek::Vec3::broadcast(f32::MIN);
    let mut min = vek::Vec3::broadcast(f32::MAX);

    for point in aabb.points() {
        // Iterate for each element in the current point and update the min/max values
        let proj = m.mul_point(point);
        proj.iter().enumerate().for_each(|(i, e)| {
            max[i] = e.max(max[i]);
            min[i] = e.min(min[i]);
        })
    }

    AABB { min, max }
}

// The 3D scene renderer
fn run(world: &mut World) {
    let global = world.resources.get::<crate::resources::WorldData>().unwrap();
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
        if update | world.pipeline.window().changed() {
            camera.update_projection_matrix(world.pipeline.window().dimensions().w as f32, world.pipeline.window().dimensions().h as f32);
            camera.update_view_matrix(position, forward, up);
        }

        // Rendering camera settings
        let camera = RenderingCamera {
            position,
            rotation,
            forward,
            view: camera.view,
            proj: camera.projection,
            clips: camera.clips,

            // Math moment
            proj_view: camera.projection * camera.view,
        };
        *world.pipeline.camera_mut() = camera;
    }

    // A bit of trolling yea?
    let query = world.ecs.try_query::<(&mut Transform, &Light)>();
    for (transform, _) in query {
        transform.rotation.rotate_x(-0.2 * world.time.delta() * 0.4);
        transform.rotation.rotate_y(0.3 * world.time.delta() * 0.8);
    }

    // Update the matrices of renderers (and bounds), only if the transforms os said renderers were externally modified
    let a = or(modified::<Transform>(), added::<Transform>());
    let b = or(modified::<Renderer>(), added::<Renderer>());
    let query = world.ecs.try_query_with::<(&Transform, &mut Renderer), _>(or(a, b));
    for (transform, renderer) in query {
        // Update the matrix if we need to
        renderer.matrix = transform.transform_matrix();
        renderer.flags.insert(RendererFlags::MATRIX_UPDATE);

        // Update the AABB bounds by using the mesh bounds
        let mesh = world.pipeline.get(&renderer.mesh);
        renderer.bounds = mesh.map(|mesh| project_aabb(mesh.bounds(), renderer.matrix)).unwrap_or_default();
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
                aabb: &renderer.bounds,
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
        if f.contains(RendererFlags::VISIBLE) {
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
        normal: models,
        shadowed: shadowed,
        lights: lights,
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
*/
