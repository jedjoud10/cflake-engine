use super::{Camera, Renderer, SceneSettings};
use crate::{
    buffer::BufferMode,
    context::{Context, GraphicsSetupSettings, Window},
    material::{AlbedoMap, MaskMap, Material, NormalMap, Pipeline, Sky, Standard},
    mesh::{Mesh, MeshImportMode, MeshImportSettings, Surface},
    prelude::{
        Filter, MipMaps, Ranged, Sampling, Texel, Texture, Texture2D, TextureImportSettings,
        TextureMode, Wrap, RG, RGB, RGBA,
    },
    shader::Shader,
};

use assets::Assets;
use ecs::{added, modified, or, EcsManager};
use glutin::{event::WindowEvent, event_loop::EventLoop};
use math::Transform;

use world::{Events, Init, Stage, Storage, Update, World};

// This event will initialize a new graphics context and create the valid window
// This will be called at the very start of the init of the engine
fn init(world: &mut World, settings: GraphicsSetupSettings, el: &EventLoop<()>) {
    let (mut window, mut context) = crate::context::new(settings, el);
    let ctx = &mut context;

    let (albedo_maps, normal_maps, mask_maps, meshes, assets) = world
        .get_mut::<(
            &mut Storage<AlbedoMap>,
            &mut Storage<NormalMap>,
            &mut Storage<MaskMap>,
            &mut Storage<Mesh>,
            &mut Assets,
        )>()
        .unwrap();

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
        mode: MeshImportMode::Static,
        generate_tangents: true,
        scale: 1.0,
    };

    // Load the default cube and sphere meshes
    /*
    let cube = assets
        .load_with::<Mesh>("engine/meshes/cube.obj", (ctx, import))
        .unwrap();

    let sphere = assets
        .load_with::<Mesh>("engine/meshes/sphere.obj", (ctx, import))
        .unwrap();
    */
    // Insert the meshes and get their handles
    let cube = meshes.insert(todo!());
    let sphere = meshes.insert(todo!());

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
    );

    // Insert the newly created resources
    world.insert(scene);
    world.insert(window);
    world.insert(context);
}

// This event will create the main skysphere and pre-register the pipelines
fn postinit(world: &mut World) {
    let (assets, ctx, settings, textures, shaders, sky_mats, ecs) = world
        .get_mut::<(
            &mut Assets,
            &mut Context,
            &mut SceneSettings,
            &mut Storage<AlbedoMap>,
            &mut Storage<Shader>,
            &mut Storage<Sky>,
            &mut EcsManager,
        )>()
        .unwrap();

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
    let texture = textures.insert(
        assets
            .load_with::<AlbedoMap>("engine/textures/sky_gradient.png", (ctx, import_settings))
            .unwrap(),
    );

    // Create the default sky material
    let material = Sky {
        gradient: texture,
        offset: 0.0,
        sun_intensity: 10.0,
        sun_radius: 1.0,
        cloud_coverage: 0.0,
        cloud_speed: 0.0,
    };

    // Get material handle and PipeId
    let material = sky_mats.insert(material);
    let pipeid = ctx.pipeline::<Sky>(shaders, assets);

    // Create a default skysphere that is pretty large
    let renderer = Renderer::default();
    let surface = Surface::new(settings.sphere(), material, pipeid);

    // Insert it as a new entity
    /*
    ecs.insert((
        renderer,
        surface,
        Transform::default().scaled(vek::Vec3::one() * 5000.0),
    )).unwrap();
    */
}

// Rendering event that will try to render the 3D scene each frame
// This will also update the world matrices of each renderer
fn rendering(world: &mut World) {
    let (ecs, ctx, settings) = world
        .get_mut::<(&mut EcsManager, &mut Context, &SceneSettings)>()
        .unwrap();

    // Don't render if we don't have a camera or a main directional light
    if !settings.can_render() {
        return;
    }

    // Update the world matrices of renderer
    let filter = or(modified::<Transform>(), added::<Transform>());
    let query = ecs
        .query_with::<(&mut Renderer, &Transform)>(filter)
        .unwrap();
    for (renderer, transform) in query {
        renderer.set_matrix(transform.matrix());
    }

    // Render all the surfaces using their respective pipelines
    ctx.extract_pipelines().into_iter().for_each(|pipe| {
        let stats = pipe.render(world);
        //dbg!(stats);
    });
}

// Window event for updating the current main canvas and world state if needed
fn window(world: &mut World, event: &mut WindowEvent) {
    match event {
        WindowEvent::Resized(size) => {
            // We might get null dimensions when the user minimizes the window
            if size.height == 0 || size.width == 0 {
                return;
            }

            // Resize the main window canvas when we resize the window
            let window = world.get_mut::<&mut Window>().unwrap();
            window
                .canvas_mut()
                .resize(vek::Extent2::new(size.width as u16, size.height as u16));
        }
        WindowEvent::CloseRequested => {
            // Stop the game engine
            *world.get_mut::<&mut world::State>().unwrap() = world::State::Stopped;
        }
        _ => {}
    }
}

// Frame startup (clearing the frame at the start of the frame)
fn clear(world: &mut World) {
    let window = world.get_mut::<&mut Window>().unwrap();
    window
        .canvas_mut()
        .clear(Some(vek::Rgb::black()), Some(1.0), None);
}

// Frame cleanup event that will just swap the front and back buffers of the current context
fn swap(world: &mut World) {
    let ctx = world.get_mut::<&mut Context>().unwrap();
    ctx.raw().swap_buffers().unwrap();
}

// Update event that will update the view matrix of the main perspective camera
// The main camera entity is stored in the Scene renderer
fn main_camera(world: &mut World) {
    // Get the ecs, window, and scene renderer
    let (ecs, scene) = world
        .get_mut::<(&mut EcsManager, &mut SceneSettings)>()
        .unwrap();

    // Fetch the main perspective camera from the scene renderer
    if let Some(entity) = scene.main_camera() {
        let mut entry = ecs.entry_mut(entity).unwrap();

        // Fetch it's components, and update them
        let (camera, transform) = entry
            .as_query::<(&mut Camera, &mut Transform)>()
            .unwrap();
        camera.update(transform);
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

    // Insert post init event
    events
        .registry::<Init>()
        .insert_with(
            postinit,
            Stage::new("graphics post init")
                .after("graphics insert")
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
            .after("user")
            .before("post user"),
    )
    .unwrap();

    // Insert scene renderer event
    reg.insert_with(
        rendering,
        Stage::new("scene rendering")
            .after("main camera update")
            .after("post user"),
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
