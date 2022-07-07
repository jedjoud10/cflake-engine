use std::num::NonZeroU8;

use cflake_engine::prelude::*;

const ASSETS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/assets/");

// Create a game that will draw a simple mesh onto the screen and a movable camera
fn main() {
    App::default()
        .set_window_title("cflake engine mesh example")
        .set_user_assets_folder_path(ASSETS_PATH)
        .insert_system(system)
        .execute();
}

// This is an init event that will be called at the start of the game
fn init(world: &mut World) {
    let (ecs, ctx, settings, keyboard, standard_mats, sky_mats, normal_maps, albedo_maps, assets, shaders) = world
        .get_mut::<(
            &mut EcsManager,
            &mut Context,
            &mut SceneSettings,
            &mut Keyboard,
            &mut Storage<Standard>,
            &mut Storage<Sky>,
            &mut Storage<NormalMap>,
            &mut Storage<AlbedoMap>,
            &mut Assets,
            &mut Storage<Shader>,
        )>()
        .unwrap();

    // Create a perspective camera and insert it into the world as an entity (and update the scene settings)
    let camera = Camera::new(90.0, 0.003, 10000.0, 16.0 / 9.0);
    let camera = ecs.insert((camera, Transform::default())).unwrap();
    settings.set_main_camera(camera);

    // We will also register some new keybinds for the camera controller
    keyboard.bind("forward", Key::W);
    keyboard.bind("backward", Key::S);
    keyboard.bind("left", Key::A);
    keyboard.bind("right", Key::D);

    // Load the persistent textures like the debug texture and missing texture
    let params = (
        Sampling {
            filter: Filter::Linear,
            wrap: Wrap::Repeat,
        },
        MipMaps::AutomaticAniso {
            samples: NonZeroU8::new(4).unwrap(),
        },
        TextureMode::Static,
    );

    let params2 = (
        Sampling {
            filter: Filter::Linear,
            wrap: Wrap::ClampToEdge,
        },
        MipMaps::Disabled,
        TextureMode::Static,
    );

    let texture = assets
        .load_with::<AlbedoMap>(
            "engine/textures/sky_gradient.png",
            (ctx, params2.0, params2.1, params2.2),
        )
        .unwrap();
    let texture = albedo_maps.insert(texture);

    let material = Sky {
        gradient: texture,
        offset: 0.0,
        sun_intensity: 10.0,
        sun_radius: 1.0,
        cloud_coverage: 0.0,
        cloud_speed: 0.0,
    };
    let material = sky_mats.insert(material);

    let pipeid = ctx.pipeline::<Sky>(shaders, assets);

    let renderer = Renderer::default();
    let surface = Surface::new(settings.sphere(), material, pipeid);
    ecs.insert((renderer, surface, Transform::default().scaled(vek::Vec3::one() * 5000.0)))
        .unwrap();

    let texture = assets
        .load_with::<NormalMap>(
            "user/textures/normal.png",
            (ctx, params.0, params.1, params.2),
        )
        .unwrap();
    let texture = normal_maps.insert(texture);

    let material = Standard::builder()
        .with_normal(&texture)
        .with_bumpiness(1.4)
        .build();
    let material = standard_mats.insert(material);

    let pipeid = ctx.pipeline::<Standard>(shaders, assets);

    let renderer = Renderer::default();
    let surface = Surface::new(settings.cube(), material, pipeid);
    ecs.insert((renderer, surface, Transform::default()))
        .unwrap();

    // Create a directional light insert it as a light entity (and update the scene settings)
    let light = Directional::default();
    let entity = ecs
        .insert((light, Transform::rotation_x(45f32.to_radians())))
        .unwrap();
    settings.set_main_directional_light(entity);
}

// We will use this update event to move the camera around
fn update(world: &mut World) {
    let (ecs, scene, keyboard, mouse, window, time) = world
        .get_mut::<(
            &mut EcsManager,
            &SceneSettings,
            &Keyboard,
            &Mouse,
            &mut Window,
            &Time,
        )>()
        .unwrap();

    // Lock the cursor basically
    window.raw().set_cursor_grab(true).unwrap();
    window.raw().set_cursor_visible(false);

    if let Some(mut entry) = scene.main_camera().and_then(|c| ecs.try_mut_entry(c)) {
        let transform = entry.get_mut::<Transform>().unwrap();
        let mut velocity = vek::Vec3::<f32>::zero();
        let forward = transform.forward();
        let right = transform.right();
        if keyboard.held("forward") {
            velocity += forward;
        } else if keyboard.held("backward") {
            velocity += -forward;
        }

        if keyboard.held("left") {
            velocity += -right;
        } else if keyboard.held("right") {
            velocity += right;
        }

        transform.position += velocity * time.delta_f32() * 10.0;

        let pos = mouse.position();
        const SENSIVITY: f32 = 0.0007;
        let rot = vek::Quaternion::rotation_y(-pos.x as f32 * SENSIVITY)
            * vek::Quaternion::rotation_x(-pos.y as f32 * SENSIVITY);
        transform.rotation = rot;
    }
}

// This is an example system that will register specific events
fn system(events: &mut Events) {
    events.registry::<Init>().insert(init);
    events.registry::<Update>().insert(update);
}
