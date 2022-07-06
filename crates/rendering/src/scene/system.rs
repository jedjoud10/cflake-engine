use super::{Camera, Renderer, SceneSettings};
use crate::{
    context::{Context, Graphics, GraphicsSetupSettings},
    material::{AlbedoMap, MaskMap, Material, NormalMap, Standard, StandardBuilder},
    mesh::SubMesh,
    prelude::{
        Filter, MipMaps, Ranged, Sampling, Texel, Texture, Texture2D, TextureMode, Wrap, RG, RGB,
        RGBA,
    },
    shader::Shader,
};

use assets::Assets;
use ecs::{added, and, contains, modified, or, Component, EcsManager, Entity};
use glutin::{event::WindowEvent, event_loop::EventLoop};
use math::Transform;
use time::Time;
use world::{Events, Init, Stage, Storage, Update, World};

// This event will initialize a new graphics context and create the valid window
// This will be called at the very start of the init of the engine
fn init(world: &mut World, settings: GraphicsSetupSettings, el: &EventLoop<()>) {
    // During initialization, the world always contains the Init resource
    // This resource contains the global event loop and all informations that are needed for

    // Create a new graphics pipeline and insert it
    let Graphics(_device, ctx) = world.entry().or_insert(Graphics::new(settings, el));

    // This function creates a 1x1 Texture2D with default settings that we can store within the scene renderer
    fn create<T: Texel>(ctx: &mut Context, texel: T::Storage) -> Texture2D<T> {
        Texture2D::<T>::new(
            ctx,
            TextureMode::Static,
            vek::Extent2::one(),
            Sampling::new(Filter::Nearest, Wrap::Repeat),
            MipMaps::Disabled,
            &[texel],
        )
        .unwrap()
    }

    // Create the default black texture
    let black = create::<RGBA<Ranged<u8>>>(ctx, vek::Vec4::zero());

    // Create the default white texture
    let white = create::<RGBA<Ranged<u8>>>(ctx, vek::Vec4::broadcast(255));

    // Create the default PBR textures (normal map, mask map)
    let normal_map = create::<RGB<Ranged<u8>>>(ctx, vek::Vec3::new(128, 128, 255));
    let mask_map = create::<RG<Ranged<u8>>>(ctx, vek::Vec2::new(255, 51));

    // Insert all of the textures into their corresponding storages
    let (albedo_maps, normal_maps, mask_maps, assets, Graphics(_, ctx)) = world
        .get_mut::<(
            &mut Storage<AlbedoMap>,
            &mut Storage<NormalMap>,
            &mut Storage<MaskMap>,
            &mut Assets,
            &mut Graphics,
        )>()
        .unwrap();

    // Convert the texture maps into texture map handles
    let black = albedo_maps.insert(black);
    let white = albedo_maps.insert(white);
    let normal_map = normal_maps.insert(normal_map);
    let mask_map = mask_maps.insert(mask_map);

    // Load the persistent textures like the debug texture and missing texture
    let params = (
        Sampling::new(Filter::Linear, Wrap::Repeat),
        MipMaps::Disabled,
        TextureMode::Static,
    );
    let debug = assets
        .load_with::<NormalMap>(
            "engine/textures/bumps.png",
            (ctx, params.0, params.1, params.2),
        )
        .unwrap();
    let missing = assets
        .load_with::<AlbedoMap>(
            "engine/textures/missing.png",
            (ctx, params.0, params.1, params.2),
        )
        .unwrap();

    // Convert them to map handles
    let missing = albedo_maps.insert(missing);
    let debug = normal_maps.insert(debug);

    // Load the default PBR material (refetch the resources since we need storage and asset loader)
    let (Graphics(_, ctx), assets, materials, shaders, submeshes) = world
        .get_mut::<(
            &mut Graphics,
            &mut Assets,
            &mut Storage<Standard>,
            &mut Storage<Shader>,
            &mut Storage<SubMesh>,
        )>()
        .unwrap();

    // Create le default material
    let material = Standard::builder(ctx, assets, shaders)
        .with_albedo(&white)
        .with_normal(&debug)
        .with_mask(&mask_map)
        .with_metallic(0.2)
        .with_roughness(1.0)
        .build();
    let material = materials.insert(material);
    ctx.register_pipeline::<Standard>(assets, shaders);

    // Load the default cube and sphere meshes
    let cube = assets
        .load_with::<SubMesh>("engine/meshes/cube.obj", ctx)
        .unwrap();

    // Insert the meshes and get their handles
    let cube = submeshes.insert(cube);

    // Create the new scene renderer from these values and insert it into the world
    let scene = SceneSettings::new(
        black,
        white.clone(),
        white,
        normal_map,
        mask_map,
        missing,
        debug,
        material,
        cube.clone(),
        cube,
    );
    world.insert(scene);
}

// Rendering event that will try to render the 3D scene each frame
// This will also update the world matrices of each renderer
fn rendering(world: &mut World) {
    let (ecs, graphics, settings) = world
        .get_mut::<(&mut EcsManager, &mut Graphics, &SceneSettings)>()
        .unwrap();
    let Graphics(_device, context) = graphics;

    if !settings.can_render() {
        return;
    }

    // Update the world matrices
    let filter = or(modified::<Transform>(), added::<Transform>());
    let query = ecs
        .try_query_with::<(&mut Renderer, &Transform)>(filter)
        .unwrap();
    for (renderer, transform) in query {
        renderer.set_matrix(transform.matrix());
    }

    // Render all the surfaces using their respective pipelines
    let pipes = context.extract_pipelines();
    let _stats = pipes
        .into_iter()
        .map(|pipe| pipe.render(world))
        .collect::<Vec<_>>();
}

// Window event for updating the current main canvas and world state if needed
fn window(world: &mut World, event: &mut WindowEvent) {
    match event {
        WindowEvent::Resized(size) => {
            // We might get null dimensions when the user minimizes the window
            if size.height == 0 || size.width == 0 {
                return;
            }

            // Resize the main device canvas when we resize the window
            let Graphics(device, _) = world.get_mut::<&mut Graphics>().unwrap();
            device
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
    let Graphics(device, _) = world.get_mut::<&mut Graphics>().unwrap();
    device
        .canvas_mut()
        .clear(Some(vek::Rgb::black()), Some(1.0), None);
}

// Frame cleanup event that will just swap the front and back buffers of the current context
fn swap(world: &mut World) {
    let Graphics(_, ctx) = world.get_mut::<&mut Graphics>().unwrap();
    ctx.raw().swap_buffers().unwrap();
}

// Update event that will update the view matrix of the main perspective camera
// The main camera entity is stored in the Scene renderer
fn main_camera(world: &mut World) {
    // Get the ecs, window, and scene renderer
    let (ecs, Graphics(_device, _), scene) = world
        .get_mut::<(&mut EcsManager, &Graphics, &mut SceneSettings)>()
        .unwrap();

    // Fetch the main perspective camera from the scene renderer
    if let Some(entity) = scene.main_camera() {
        let mut entry = ecs.try_mut_entry(entity).unwrap();

        // Fetch it's components, and update them
        let (camera, transform) = entry
            .get_mut_layout::<(&mut Camera, &mut Transform)>()
            .unwrap();
        camera.update(transform);
    }
}

// Main rendering/graphics system that will register the appropriate events
pub fn system(events: &mut Events, settings: GraphicsSetupSettings) {
    // Insert init events
    events
        .registry::<Init>()
        .insert_with(
            |world: &mut World, el: &EventLoop<()>| init(world, settings, el),
            Stage::new("graphics insert").after("asset loader insert"),
        )
        .unwrap();

    // Insert update events (fetch the registry)
    let reg = events.registry::<Update>();
    reg.insert_with(clear, Stage::new("window clear").before("user"))
        .unwrap();
    reg.insert_with(
        main_camera,
        Stage::new("main camera update")
            .after("user")
            .before("post user"),
    )
    .unwrap();
    reg.insert_with(
        rendering,
        Stage::new("scene rendering")
            .after("main camera update")
            .after("post user"),
    )
    .unwrap();
    reg.insert_with(
        swap,
        Stage::new("window back buffer swap").after("scene rendering"),
    )
    .unwrap();

    // Insert window event
    events.registry::<WindowEvent>().insert(window);
}
