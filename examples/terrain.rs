use cflake_engine::prelude::*;

// Terrain example game window
fn main() {
    App::default()
        .set_app_name("cflake engine terrain example")
        .set_window_fullscreen(true)
        //.set_frame_rate_limit(FrameRateLimit::VSync)
        //set_frame_rate_limit(FrameRateLimit::Limited(120))
        //.set_logging_level(LevelFilter::Trace)
        .insert_init(init)
        .insert_update(update)
        .execute();
}

// Creates a movable camera, sky entity, and procedural terrain
fn init(world: &mut World) {
    let graphics = world.get::<Graphics>().unwrap();
    let assets = world.get::<Assets>().unwrap();

    // Setup the assets that will be loaded
    asset!(assets, "user/textures/diffuse1.jpg", "/examples/assets/");
    asset!(assets, "user/textures/normal1.jpg", "/examples/assets/");
    asset!(assets, "user/textures/mask1.jpg", "/examples/assets/");
    asset!(assets, "user/textures/diffuse5.jpg", "/examples/assets/");
    asset!(assets, "user/textures/normal5.jpg", "/examples/assets/");
    asset!(assets, "user/textures/mask5.jpg", "/examples/assets/");

    // Create the terrain generator's settings
    let settings = TerrainSettings::new(
        &graphics,
        64,
        false,
        false,
        4,
        1024,
        7,
        0.4f32,
        Some(&[
            TerrainSubMaterial {
                diffuse: "user/textures/diffuse5.jpg".to_string(),
                normal: "user/textures/normal5.jpg".to_string(),
                mask: "user/textures/mask5.jpg".to_string(),
            },
            TerrainSubMaterial {
                diffuse: "user/textures/diffuse1.jpg".to_string(),
                normal: "user/textures/normal1.jpg".to_string(),
                mask: "user/textures/mask1.jpg".to_string(),
            },
        ]),
    )
    .unwrap();

    // Drop (needed) to insert settings
    drop(graphics);
    drop(assets);
    world.insert(settings);

    // Create a movable camera
    let mut scene = world.get_mut::<Scene>().unwrap();
    scene.insert((
        Position::default(),
        Rotation::default(),
        Velocity::default(),
        Camera::default(),
        ChunkViewer::default(),
        CameraController::default(),
    ));

    // Create a directional light
    let light = DirectionalLight {
        color: vek::Rgb::one() * 3.6,
    };
    let rotation = vek::Quaternion::rotation_x(-15.0f32.to_radians()).rotated_y(45f32.to_radians());
    scene.insert((light, Rotation::from(rotation)));

    let mut pbrs = world.get_mut::<Storage<PhysicallyBasedMaterial>>().unwrap();

    // Create a new material instance
    let material = pbrs.insert(PhysicallyBasedMaterial {
        albedo_map: None,
        normal_map: None,
        mask_map: None,
        bumpiness: 1.0,
        roughness: 1.0,
        metallic: 1.0,
        ambient_occlusion: 3.0,
        tint: vek::Rgb::white(),
        scale: vek::Extent2::one(),
    });

    let pipelines = world.get::<Pipelines>().unwrap();
    let id = pipelines.get::<PhysicallyBasedMaterial>().unwrap();
    let renderer = world.get::<ForwardRenderer>().unwrap();
    let sphere = renderer.sphere.clone();
    let renderer = Renderer::default();
    let position = Position::default();
    let surface = Surface::new(sphere.clone(), material.clone(), id.clone());
    scene.prefabify("sphere", (renderer, position, surface));
}

// Updates the light direction and quites from the engine
fn update(world: &mut World) {
    let time = world.get::<Time>().unwrap();
    let mut state = world.get_mut::<State>().unwrap();
    let time = &*time;
    let input = world.get::<Input>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();

    // Rotation the light
    if let Some((rotation, _)) = scene.find_mut::<(&mut Rotation, &DirectionalLight)>() {
        rotation.rotate_y(-0.1 * time.delta().as_secs_f32());
    }

    // Exit the game when the user pressed Escape
    if input.get_button(KeyboardButton::Escape).pressed() {
        *state = State::Stopped;
    }

    // Create a new sphere in front of the camera when we press the right mouse button
    if input.get_button(MouseButton::Right).pressed() {
        let (_, position, rotation) = scene.find::<(&Camera, &Position, &Rotation)>().unwrap();
        let mut entry = scene.instantiate("sphere").unwrap();
        **entry.get_mut::<Position>().unwrap() = rotation.forward() * 3.0 + **position;
    }
}
