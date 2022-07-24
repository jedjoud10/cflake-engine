use super::{Camera, Renderer, SceneSettings};
use crate::{
    buffer::BufferMode,
    context::{Context, GraphicsSetupSettings, Window},
    material::{AlbedoMap, MaskMap, Material, NormalMap, Pipeline, PipelineStats, Sky, Standard},
    mesh::{Mesh, MeshImportSettings, Surface},
    prelude::{
        Filter, MipMaps, Ranged, Sampling, Texel, Texture, Texture2D, TextureImportSettings,
        TextureMode, Wrap, RG, RGB, RGBA, R, Depth,
    },
    shader::Shader, canvas::{Canvas, ColorAttachment, DepthAttachment, ToCanvasAttachment},
};

use assets::Assets;
use ecs::{added, modified, or, EcsManager};
use glutin::{event::WindowEvent, event_loop::EventLoop};
use math::{Scale, Location, Rotation, IntoMatrix};
use world::{Events, Init, Stage, Storage, Update, World};

// This event will initialize a new graphics context and create the valid window
// This will be called at the very start of the init of the engine
fn init(world: &mut World, settings: GraphicsSetupSettings, el: &EventLoop<()>) -> (Window, Context, SceneSettings) {
    // Insert the default storages
    world.insert(Storage::<AlbedoMap>::default());
    world.insert(Storage::<NormalMap>::default());
    world.insert(Storage::<MaskMap>::default());
    world.insert(Storage::<Mesh>::default());
    world.insert(Storage::<Shader>::default());
    world.insert(Storage::<Standard>::default());
    world.insert(Storage::<Sky>::default());
    world.insert(Storage::<Canvas>::default());
    world.insert(Storage::<ColorAttachment>::default());
    world.insert(Storage::<DepthAttachment>::default());

    // Get mutable references to the data that we must use
    let mut albedo_maps = world.get_mut::<Storage<AlbedoMap>>().unwrap();
    let mut normal_maps = world.get_mut::<Storage<NormalMap>>().unwrap();
    let mut mask_maps = world.get_mut::<Storage<MaskMap>>().unwrap();
    let mut meshes = world.get_mut::<Storage<Mesh>>().unwrap();
    let mut shaders = world.get_mut::<Storage<Shader>>().unwrap();
    let mut standard_materials = world.get_mut::<Storage<Standard>>().unwrap();
    let mut sky_materials = world.get_mut::<Storage<Sky>>().unwrap();
    let mut assets = world.get_mut::<Assets>().unwrap();
    let mut ecs = world.get_mut::<EcsManager>().unwrap();
    let mut canvases = world.get_mut::<Storage<Canvas>>().unwrap();
    let mut color_attachments = world.get_mut::<Storage<ColorAttachment>>().unwrap();
    let mut depth_attachments = world.get_mut::<Storage<DepthAttachment>>().unwrap();

    // Create the window and graphical context
    let (mut window, mut context) = crate::context::new(settings, el);
    let ctx = &mut context;

    // This function creates a 1x1 Texture2D with default settings that we can store within the scene renderer
    fn create<T: Texel>(ctx: &mut Context, texel: T::Storage) -> Texture2D<T> {
        Texture2D::<T>::new(
            ctx,
            TextureMode::Static,
            vek::Extent2::one(),
            Sampling {
                filter: Filter::Nearest,
                wrap: Wrap::Repeat,
            },
            MipMaps::Disabled,
            &[texel],
        )
        .unwrap()
    }

    // Create the 1x1 default textures
    let black = create::<RGBA<Ranged<u8>>>(ctx, vek::Vec4::zero());
    let white = create::<RGBA<Ranged<u8>>>(ctx, vek::Vec4::broadcast(255));
    let normal_map = create::<RGB<Ranged<u8>>>(ctx, vek::Vec3::new(128, 128, 255));
    let mask_map = create::<RG<Ranged<u8>>>(ctx, vek::Vec2::new(255, 51));

    // Convert the texture maps into texture map handles
    let black = albedo_maps.insert(black);
    let white = albedo_maps.insert(white);
    let normal_map = normal_maps.insert(normal_map);
    let mask_map = mask_maps.insert(mask_map);

    // Load the persistent textures like the debug texture and missing texture
    let settings = TextureImportSettings {
        sampling: Sampling {
            filter: Filter::Nearest,
            wrap: Wrap::Repeat,
        },
        mode: TextureMode::Static,
        mipmaps: MipMaps::Automatic,
    };

    let debug = assets
        .load_with::<NormalMap>("engine/textures/bumps.png", (ctx, settings))
        .unwrap();
    let missing = assets
        .load_with::<AlbedoMap>("engine/textures/missing.png", (ctx, settings))
        .unwrap();

    // Convert them to map handles
    let missing = albedo_maps.insert(missing);
    let debug = normal_maps.insert(debug);

    let import = MeshImportSettings {
        mode: BufferMode::Static,
        generate_normals: false,
        generate_tangents: false,
        scale: 1.0,
    };

    // Load the default cube and sphere meshes
    let cube = assets
        .load_with::<Mesh>("engine/meshes/cube.obj", (ctx, import))
        .unwrap();
    let sphere = assets
        .load_with::<Mesh>("engine/meshes/sphere.obj", (ctx, import))
        .unwrap();
    
    // Insert the meshes and get their handles
    let cube = meshes.insert(cube);
    let sphere = meshes.insert(sphere);

    // Settings for framebuffer textures
    let sampling = Sampling {
        filter: Filter::Linear,
        wrap: Wrap::ClampToEdge,
    };
    let mipmaps = MipMaps::Disabled;

    // Create the render color texture
    let color: ColorAttachment = <ColorAttachment as Texture>::new(ctx, TextureMode::Resizable, window.canvas().size(), sampling, mipmaps, &[]).unwrap();
    let color = color_attachments.insert(color);
    let t1 = (&*color_attachments, color);

    // Create the render depth texture
    let depth: DepthAttachment = <DepthAttachment as Texture>::new(ctx, TextureMode::Resizable, window.canvas().size(), sampling, mipmaps, &[]).unwrap();
    let depth = depth_attachments.insert(depth);
    let t2 = (&*depth_attachments, depth);
    
    // Create the canvas that we will draw our 3D objects onto
    let targets: Vec<&dyn ToCanvasAttachment> = vec![&t1, &t2];
    let canvas = Canvas::new(ctx, window.canvas().size(), targets).unwrap();
    let canvas = canvases.insert(canvas);
    
    // Create the new scene renderer from these values and insert it into the world
    let scene = SceneSettings::new(
        black,
        white.clone(),
        white,
        normal_map,
        mask_map,
        missing,
        debug,
        cube,
        sphere,
        canvas,
    );

    // Sky gradient texture import settings
    let import_settings = TextureImportSettings {
        sampling: Sampling {
            filter: Filter::Linear,
            wrap: Wrap::ClampToEdge,
        },
        mode: TextureMode::Static,
        mipmaps: MipMaps::Disabled,
    };

    // Load in the texture
    let texture = albedo_maps.insert(
        assets
            .load_with::<AlbedoMap>("engine/textures/sky_gradient.png", (ctx, import_settings))
            .unwrap(),
    );

    // Create the default sky material
    let material = Sky {
        gradient: texture,
        sun_intensity: 15.0,
        sun_size: 1.05,
        cloud_coverage: 0.0,
        cloud_speed: 0.0,
    };

    // Create the default Standard material pipeline
    ctx.pipeline::<Standard>(&mut shaders, &mut assets);

    // Create the default Sky material pipeline and default Sky sphere surface
    let material = sky_materials.insert(material);
    let pipeid = ctx.pipeline::<Sky>(&mut shaders, &mut assets);
    let renderer = Renderer::default();
    let surface = Surface::new(scene.sphere(), material, pipeid);

    // Insert it as a new entity
    ecs.insert((
        renderer,
        surface,
        Scale::from(vek::Vec3::one() * 5000.0)
    ));

    (window, context, scene)
}

// Update the global mesh matrices of objects that have been modified
fn update_matrices(world: &mut World) {
    let mut ecs = world.get_mut::<EcsManager>().unwrap();
    
    // TODO: Add filter
    let query = ecs
        .query::<(&mut Renderer, Option<&Location>, Option<&Rotation>, Option<&Scale>)>()
        .unwrap();

    for (renderer, location, rotation, scale) in query {
        let mut matrix = vek::Mat4::<f32>::identity();

        if let Some(location) = location {
            matrix *= location.into_matrix();
        }

        if let Some(rotation) = rotation {
            matrix *= rotation.into_matrix();
        }

        if let Some(scale) = scale {
            matrix *= scale.into_matrix();
        }
        
        renderer.set_matrix(matrix);
    }
}

// Rendering event that will try to render the 3D scene each frame
fn rendering(world: &mut World) {
    if !world.get::<SceneSettings>().unwrap().can_render() {
        return;
    }

    let pipelines = world
        .get::<Context>()
        .unwrap()
        .extract_pipelines()
        .into_iter();

    let stats = pipelines
        .into_iter()
        .map(|p| p.render(world))
        .collect::<Vec<PipelineStats>>();
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

            // Resize the default canvas when we resize the window
            let mut window = world.get_mut::<Window>().unwrap();
            window.canvas_mut().resize(extent);

            // Resize the main rendering canvas when we resize the window
            /*
            let scene = world.get::<SceneSettings>().unwrap();
            let mut canvases = world.get_mut::<Storage<Canvas>>().unwrap();
            let canvas = &mut canvases[scene.canvas()];
            canvas.resize(extent);
            */
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
    window
        .canvas_mut()
        .clear(Some(vek::Rgb::black()), Some(1.0), None);
}

// Frame cleanup event that will just swap the front and back buffers of the current context
fn swap(world: &mut World) {
    let ctx = world.get_mut::<Context>().unwrap();
    ctx.raw().swap_buffers().unwrap();
}

// Update event that will update the view matrix of the main perspective camera
// The main camera entity is stored in the Scene renderer
fn main_camera(world: &mut World) {
    // Get the ecs, window, and scene renderer
    let mut ecs = world.get_mut::<EcsManager>().unwrap();
    let scene = world.get::<SceneSettings>().unwrap();

    // Fetch the main perspective camera from the scene renderer
    if let Some(entity) = scene.main_camera() {
        let mut entry = ecs.entry_mut(entity).unwrap();

        // Fetch it's components, and update them
        let (camera, location, rotation) = entry.as_query::<(&mut Camera, &Location, &Rotation)>().unwrap();
        camera.update(location, rotation);
    }
}

// Main rendering/graphics system that will register the appropriate events
pub fn system(events: &mut Events, settings: GraphicsSetupSettings) {
    // Insert graphics init event
    events
        .registry::<Init>()
        .insert_with(
            |world: &mut World, el: &EventLoop<()>| {
                let (window, ctx, scene) = init(world, settings, el);
                world.insert(window);
                world.insert(ctx);
                world.insert(scene);
            },
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
        Stage::new("main camera update")
            .after("post user"),
    )
    .unwrap();

    // Insert update renderer event
    reg.insert_with(
        update_matrices,
        Stage::new("update renderer matrices")
            .after("post user"),
    )
    .unwrap();

    // Insert scene renderer event
    reg.insert_with(
        rendering,
        Stage::new("scene rendering")
            .after("main camera update")
            .after("update renderer matrices")
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
