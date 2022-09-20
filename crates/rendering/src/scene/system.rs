use super::{
    Camera, ClusteredShading, Compositor, DirectionalLight, PostProcessing, RenderedFrameStats,
    Renderer, ShadowMapping,
};
use crate::{
    buffer::{BufferMode},
    context::{Context, GraphicsSetupSettings, Window},
    display::{Display, PrimitiveMode, RasterSettings},
    material::{AlbedoMap, MaskMap, NormalMap, Sky, Standard},
    mesh::{Mesh, Surface},
    prelude::{
        FragmentStage, Processor, ShaderCompiler,
        Texture, VertexStage,
    },
    shader::Shader,
};


use assets::Assets;
use ecs::Scene;
use glutin::{event::WindowEvent, event_loop::EventLoop};
use math::{IntoMatrix, Location, Rotation, Scale};
use world::{Events, Init, Stage, Storage, Update, World};

// This event will initialize a new graphics context and create the valid window
// This will be called at the very start of the init of the engine
fn init(world: &mut World, settings: GraphicsSetupSettings, el: &EventLoop<()>) {
    // Insert the default storages
    world.insert(Storage::<AlbedoMap>::default());
    world.insert(Storage::<NormalMap>::default());
    world.insert(Storage::<MaskMap>::default());
    world.insert(Storage::<Mesh>::default());
    world.insert(Storage::<Shader>::default());
    world.insert(Storage::<Standard>::default());
    world.insert(Storage::<Sky>::default());

    // Get mutable references to the data that we must use
    let mut shaders = world.get_mut::<Storage<Shader>>().unwrap();
    let mut assets = world.get_mut::<Assets>().unwrap();

    // Create the window and graphical context
    let (window, mut context) = crate::context::new(settings, el);
    let ctx = &mut context;

    // Create the clustered shading and the shadow mapper
    let clustered_shading = ClusteredShading::new(ctx, 64, &window, &mut shaders, &mut assets);
    let shadow_mapping = ShadowMapping::new(20.0, 40.0, 8192, ctx, &mut shaders, &mut assets);

    // Create the positions vec for the fullscreen quad
    let positions = vec![
        vek::Vec3::new(-1.0, -1.0, 0.0),
        vek::Vec3::new(1.0, -1.0, 0.0),
        vek::Vec3::new(-1.0, 1.0, 0.0),
        vek::Vec3::new(1.0, 1.0, 0.0),
    ];

    // Create the triangles vec for the fullscreen quad
    let triangles = vec![[0, 1, 2], [1, 3, 2]];
    let quad = Mesh::from_vecs(
        ctx,
        BufferMode::default(),
        Some(positions),
        None,
        None,
        None,
        None,
        triangles,
    )
    .unwrap();

    // Create the compositor shader
    let vertex = assets
        .load::<VertexStage>("engine/shaders/passthrough.vrsh.glsl")
        .unwrap();
    let fragment = assets
        .load::<FragmentStage>("engine/shaders/compositor.frsh.glsl")
        .unwrap();
    let shader = ShaderCompiler::link((vertex, fragment), Processor::new(&mut assets), ctx);

    // Create the internally used compositor
    let compositor = Compositor {
        quad,
        compositor: shader,
    };

    // Create the post-processing settings
    let postprocessing = PostProcessing {
        tonemapping_strength: 1.0,
        exposure: 1.2,
        gamma: 2.2,
        vignette_strength: 8.5,
        vignette_size: 0.2,
        bloom_radius: 25.0,
        bloom_strength: 1.0,
        bloom_contrast: 1.0,
    };

    // Create the frame-to-frame basis stats
    let stats = RenderedFrameStats::default();

    // Drop the old write/read handles
    drop(shaders);
    drop(assets);

    // Insert the newly made resources
    world.insert(window);
    world.insert(context);
    world.insert(clustered_shading);
    world.insert(postprocessing);
    world.insert(stats);
    world.insert(compositor);
    world.insert(shadow_mapping);
}

// Update the global mesh matrices of objects that have been modified
fn update_matrices(world: &mut World) {
    let mut ecs = world.get_mut::<Scene>().unwrap();

    // Filter the objects that have changed
    use ecs::*;
    let f1 = modified::<Location>();
    let f2 = modified::<Rotation>();
    let f3 = modified::<Scale>();
    let f4 = added::<Renderer>();
    let filter = or(or(f1, f2), or(f3, f4));
    let query = ecs
        .query_with_filter::<(
            &mut Renderer,
            Option<&Location>,
            Option<&Rotation>,
            Option<&Scale>,
        )>(filter)
        .unwrap();

    // Update the matrices of objects that might contain location, rotation, or scale
    for (renderer, location, rotation, scale) in query {
        let mut matrix = vek::Mat4::<f32>::identity();
        matrix = location.map_or(matrix, |l| matrix * l.into_matrix());
        matrix *= rotation.map_or(matrix, |r| matrix * r.into_matrix());
        matrix *= scale.map_or(matrix, |s| matrix * s.into_matrix());
        renderer.matrix = matrix;
    }
}

// Rendering event that will try to render the 3D scene each frame
fn render_surfaces(world: &mut World) {
    // Check if we can even render the scene in the first place
    let shading = world.get::<ClusteredShading>().unwrap();
    let mut old_stats = world.get_mut::<RenderedFrameStats>().unwrap();
    *old_stats = RenderedFrameStats::default();
    if shading.main_camera().is_none() || shading.main_directional_light().is_none() {
        return;
    }
    drop(old_stats);
    drop(shading);
    
    // Extract the pipelines and render them
    let pipelines = world
        .get::<Context>()
        .unwrap()
        .extract_pipelines()
        .into_iter();

    // Render the pipelines one by one
    for render in pipelines {
        render(world);
    }

    // Render the quad onto the screen now
    let mut _compositor = world.get_mut::<Compositor>().unwrap();
    let compositor = &mut *_compositor;
    let mut _shading = world.get_mut::<ClusteredShading>().unwrap();
    let shading = &mut *_shading;
    let mut _shadow_mapper = world.get_mut::<ShadowMapping>().unwrap();
    let shadow_mapper = &mut *_shadow_mapper;
    let pp = world.get::<PostProcessing>().unwrap();
    let ecs = world.get::<Scene>().unwrap();

    // Get the main window since we will draw to it
    let mut window = world.get_mut::<Window>().unwrap();
    let mut ctx = world.get_mut::<Context>().unwrap();

    // Get the renderering camera
    let camera_entity = ecs.entry(shading.main_camera.unwrap()).unwrap();
    let camera = camera_entity.get::<Camera>().unwrap();

    // Create the full screen rasterizer
    let settings = RasterSettings {
        depth_test: None,
        scissor_test: None,
        primitive: PrimitiveMode::Triangles { cull: None },
        srgb: false,
        blend: None,
    };
    let (mut rasterizer, mut uniforms) =
        window.rasterizer(&mut ctx, &mut compositor.compositor, settings);

    // Set the shading uniforms
    let resolution = vek::Vec2::<i32>::from(rasterizer.display().size().as_::<i32>());
    uniforms.set_vec2("resolution", resolution);
    uniforms.set_sampler("color", &shading.color_tex);
    uniforms.set_sampler("depth", &shading.depth_tex);
    uniforms.set_sampler("shadow_map", &shadow_mapper.depth_tex);
    uniforms.set_scalar("z_near", camera.clip_planes().x);
    uniforms.set_scalar("z_far", camera.clip_planes().y);

    // Set post processing uniforms
    uniforms.set_scalar("tonemapping_strength", pp.tonemapping_strength);
    uniforms.set_scalar("exposure", pp.exposure);
    uniforms.set_scalar("gamma", pp.gamma);
    uniforms.set_scalar("vignette_strength", pp.vignette_strength);
    uniforms.set_scalar("vignette_size", pp.vignette_size);

    // Render the screen quad
    rasterizer.draw(&compositor.quad, uniforms.validate().unwrap());
    ctx.flush();
}

// Window event for updating the current main canvas and world state if needed
fn window(world: &mut World, event: &mut WindowEvent) {
    match event {
        WindowEvent::Resized(size) => {
            // We might get null dimensions when the user minimizes the window
            let extent = if size.height > 0 && size.width > 0 {
                vek::Extent2::new(size.width as u16, size.height as u16)
            } else {
                return;
            };

            // Resize the clustered shading canvas
            let mut shading = world.get_mut::<ClusteredShading>().unwrap();
            shading.color_tex.resize(extent);
            shading.depth_tex.resize(extent);
        }
        WindowEvent::CloseRequested => {
            *world.get_mut::<world::State>().unwrap() = world::State::Stopped;
        }
        _ => {}
    }
}

// Frame startup (clearing the frame at the start of the frame)
fn clear(world: &mut World) {
    let mut window = world.get_mut::<Window>().unwrap();
    window.clear(Some(vek::Rgb::black()), Some(1.0), None);

    // Clear the screen textures
    let shading = world.get_mut::<ClusteredShading>().unwrap();
    shading
        .color_tex
        .mip_mut(0)
        .unwrap()
        .splat(vek::Vec3::zero());
    shading.depth_tex.mip_mut(0).unwrap().splat(u32::MAX);

    // Clear the shadowmap texture
    let shadow_map = world.get_mut::<ShadowMapping>().unwrap();
    shadow_map.depth_tex.mip_mut(0).unwrap().splat(u32::MAX);
}

// Frame cleanup event that will just swap the front and back buffers of the current context
fn swap(world: &mut World) {
    let ctx = world.get_mut::<Context>().unwrap();
    ctx.raw().swap_buffers().unwrap();
}

// Update event that will set/update the main perspective camera
fn main_camera(world: &mut World) {
    let mut ecs = world.get_mut::<Scene>().unwrap();
    let mut shading = world.get_mut::<ClusteredShading>().unwrap();

    // Fetch the main perspective camera from the scene renderer
    if let Some(entity) = shading.main_camera {
        // Disable the entity in the resource if it got removed
        let mut entry = if let Some(entry) = ecs.entry_mut(entity) {
            entry
        } else {
            shading.main_camera = None;
            return;
        };

        // Fetch it's components,and update them
        let (camera, location, rotation) = entry
            .as_query::<(&mut Camera, &Location, &Rotation)>()
            .unwrap();
        camera.update(location, rotation);
    } else {
        // Set the main camera if we did not find one
        let mut query = ecs
            .view_with_id::<(&Camera, &Location, &Rotation)>()
            .unwrap();
        if let Some((_, entity)) = query.next() {
            shading.main_camera = Some(entity);
        }
    }
}

// Update event that will set the main directional light
fn main_directional_light(world: &mut World) {
    let ecs = world.get_mut::<Scene>().unwrap();
    let mut shading = world.get_mut::<ClusteredShading>().unwrap();

    // Fetch the main directional light
    if let Some(entity) = shading.main_directional_light {
        // Disable the main directional shading light if it got removed
        if !ecs.contains(entity) {
            shading.main_directional_light = None;
        }
    } else {
        // Set the main directional light if we did not find one
        let mut query = ecs
            .view_with_id::<(&Rotation, &DirectionalLight)>()
            .unwrap();
        if let Some((_, entity)) = query.next() {
            shading.main_directional_light = Some(entity);
        }
    }
}

// Update event that will set the main skysphere
fn main_sky_sphere(world: &mut World) {
    let ecs = world.get_mut::<Scene>().unwrap();
    let mut shading = world.get_mut::<ClusteredShading>().unwrap();

    // Fetch the main sky sphere from the scene renderer
    if let Some(entity) = shading.skysphere_entity {
        // Disable the main directional shading light if it got removed
        if !ecs.contains(entity) {
            shading.skysphere_entity = None;
        }
    } else {
        // Set the main sky sphere if we did not find one
        let mut query = ecs.view_with_id::<(&Renderer, &Surface<Sky>)>().unwrap();
        if let Some((_, entity)) = query.next() {
            shading.skysphere_entity = Some(entity);
        }
    }
}

// Update the light positions inside the shadow mapper
fn shadow_map(world: &mut World) {
    let ecs = world.get::<Scene>().unwrap();
    let shading = world.get::<ClusteredShading>().unwrap();
    let main_light = shading.main_directional_light;
    let main_light_entry = main_light.map(|id| ecs.entry(id)).flatten();

    if let Some(entry) = main_light_entry {
        if let Ok(rotation) = entry.get::<Rotation>() {
            let mut shadow = world.get_mut::<ShadowMapping>().unwrap();
            shadow.view_matrix = vek::Mat4::look_at_rh(vek::Vec3::zero(), -rotation.forward(), -rotation.up());
        }
    }
    
}

// Main rendering/graphics system that will register the appropriate events
pub fn system(events: &mut Events, settings: GraphicsSetupSettings) {
    // Insert graphics init event
    events
        .registry::<Init>()
        .insert_with(
            |world: &mut World, el: &EventLoop<()>| init(world, settings, el),
            Stage::new("graphics insert")
                .after("asset loader insert")
                .before("user"),
        )
        .unwrap();

    // Insert update events (fetch the registry)
    let reg = events.registry::<Update>();
    reg.insert_with(clear, Stage::new("window clear").before("user"))
        .unwrap();

    // Insert camera update event
    reg.insert_with(
        main_camera,
        Stage::new("main camera update").after("post user"),
    )
    .unwrap();

    // Insert the directional light update event
    reg.insert_with(
        main_directional_light,
        Stage::new("main directional light update").after("post user"),
    )
    .unwrap();

    // Insert the directional sky sphere update event
    reg.insert_with(
        main_sky_sphere,
        Stage::new("sky sphere update").after("post user"),
    )
    .unwrap();

    // Insert update renderer event
    reg.insert_with(
        update_matrices,
        Stage::new("update renderer matrices").after("post user"),
    )
    .unwrap();

    // Insert shadow mapping update view matrix event
    reg.insert_with(
        shadow_map, 
        Stage::new("update shadow mapping view matrix")
            .after("post user")
            .before("scene rendering")
    )
    .unwrap();

    // Insert scene renderer event
    reg.insert_with(
        render_surfaces,
        Stage::new("scene rendering")
            .before("main camera update")
            .after("main directional light update")
            .after("sky sphere update")
            .after("update renderer matrices"),
    )
    .unwrap();

    // Insert window buffer swap event
    reg.insert_with(
        swap,
        Stage::new("window back buffer swap").after("scene rendering"),
    )
    .unwrap();

    // Insert window event
    events.registry::<WindowEvent>().insert(window);
}
