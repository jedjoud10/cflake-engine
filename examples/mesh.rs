use std::num::NonZeroU8;

use cflake_engine::prelude::*;

const ASSETS_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/examples/assets/");

// Create a game that will draw a simple mesh onto the screen and a movable camera
fn main() {
    App::default()
        .set_window_title("cflake engine mesh example")
        .set_user_assets_folder_path(ASSETS_PATH)
        .insert_init(init)
        .insert_update(update)
        .execute();
}

// This is an init event that will be called at the start of the game
fn init(world: &mut World) {
    let mut ecs = world.get_mut::<EcsManager>().unwrap();
    let mut ctx = world.get_mut::<Context>().unwrap();
    let mut settings = world.get_mut::<SceneSettings>().unwrap();
    let mut keyboard = world.get_mut::<Keyboard>().unwrap();
    let mut standard_materials = world.get_mut::<Storage<Standard>>().unwrap();
    let mut albedo_maps = world.get_mut::<Storage<AlbedoMap>>().unwrap();
    let mut normal_maps = world.get_mut::<Storage<NormalMap>>().unwrap();
    let mut assets = world.get_mut::<Assets>().unwrap();
    let mut shaders = world.get_mut::<Storage<Shader>>().unwrap();

    // Create a perspective camera and insert it into the world as an entity (and update the scene settings)
    let camera = Camera::new(90.0, 0.003, 10000.0, 16.0 / 9.0);
    let camera = ecs.insert((camera, Transform::default()));
    settings.set_main_camera(camera);

    // We will also register some new keybinds for the camera controller
    keyboard.bind("forward", Key::W);
    keyboard.bind("backward", Key::S);
    keyboard.bind("left", Key::A);
    keyboard.bind("right", Key::D);

    // Create a directional light insert it as a light entity (and update the scene settings)
    let light = Directional::default();
    let entity = ecs
        .insert((light, Transform::rotation_x(45f32.to_radians())));
    settings.set_main_directional_light(entity);

    // Load the persistent textures like the debug texture and missing texture
    let import_settings = TextureImportSettings {
        sampling: Sampling {
            filter: Filter::Linear,
            wrap: Wrap::Repeat,
        },
        mipmaps: MipMaps::AutomaticAniso {
            samples: NonZeroU8::new(4).unwrap(),
        },
        mode: TextureMode::Static,
    };

    let texture = assets
        .load_with::<NormalMap>("user/textures/normal.png", (&mut ctx, import_settings))
        .unwrap();
    let texture = normal_maps.insert(texture);

    let material = Standard::builder()
        .with_normal(&texture)
        .with_bumpiness(1.4)
        .build();
    let material = standard_materials.insert(material);

    let pipeid = ctx.pipeline::<Standard>(&mut shaders, &mut assets);

    let renderer = Renderer::default();
    let surface = Surface::new(settings.cube(), material, pipeid);
    ecs.insert((renderer, surface, Transform::default()));
}

// We will use this update event to move the camera around
fn update(world: &mut World) {
    let mut ecs = world.get_mut::<EcsManager>().unwrap();
    let settings = world.get::<SceneSettings>().unwrap();
    let keyboard = world.get::<Keyboard>().unwrap();
    let mouse = world.get::<Mouse>().unwrap();
    let window = world.get_mut::<Window>().unwrap();
    let time = world.get::<Time>().unwrap();

    // Lock the cursor basically
    window.raw().set_cursor_grab(true).unwrap();
    window.raw().set_cursor_visible(false);
    
    if let Some(mut entry) = settings.main_camera().and_then(|c| ecs.entry_mut(c)) {
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