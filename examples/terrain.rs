use cflake_engine::prelude::*;

// Terrain example game window
fn main() {
    App::default()
        .set_app_name("cflake engine terrain example")
        .set_window_fullscreen(true)
        //.set_frame_rate_limit(FrameRateLimit::VSync)
        //.set_frame_rate_limit(FrameRateLimit::Limited(120))
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
    asset!(assets, "user/textures/diffuse.jpg", "/examples/assets/");
    asset!(assets, "user/textures/normal.jpg", "/examples/assets/");
    asset!(assets, "user/textures/mask.jpg", "/examples/assets/");
    asset!(assets, "user/textures/diffuse1.jpg", "/examples/assets/");
    asset!(assets, "user/textures/normal1.jpg", "/examples/assets/");
    asset!(assets, "user/textures/mask1.jpg", "/examples/assets/");
    asset!(assets, "user/textures/diffuse2.jpg", "/examples/assets/");
    asset!(assets, "user/textures/normal2.jpg", "/examples/assets/");
    asset!(assets, "user/textures/mask2.jpg", "/examples/assets/");
    asset!(assets, "user/textures/diffuse3.jpg", "/examples/assets/");
    asset!(assets, "user/textures/normal3.jpg", "/examples/assets/");
    asset!(assets, "user/textures/mask3.jpg", "/examples/assets/");
    asset!(assets, "user/textures/diffuse4.jpg", "/examples/assets/");
    asset!(assets, "user/textures/normal4.jpg", "/examples/assets/");
    asset!(assets, "user/textures/mask4.jpg", "/examples/assets/");

    // Create the terrain generator's settings
    let settings = TerrainSettings::new(
        &graphics,
        64,
        false,
        false,
        8,
        1024,
        6,
        None,
        /*
        Some(&[
            TerrainSubMaterial {
                diffuse: "user/textures/diffuse3.jpg".to_string(),
                normal: "user/textures/normal3.jpg".to_string(),
                mask: "user/textures/mask3.jpg".to_string(),
            },
            TerrainSubMaterial {
                diffuse: "user/textures/diffuse4.jpg".to_string(),
                normal: "user/textures/normal4.jpg".to_string(),
                mask: "user/textures/mask4.jpg".to_string(),
            },
            TerrainSubMaterial {
                diffuse: "user/textures/diffuse2.jpg".to_string(),
                normal: "user/textures/normal2.jpg".to_string(),
                mask: "user/textures/mask2.jpg".to_string(),
            },
        ]),
        */
    )
    .unwrap();

    // Drop (needed) to insert settings
    drop(graphics);
    drop(assets);
    world.insert(settings);

    // Fetch the required resources from the world
    let assets = world.get::<Assets>().unwrap();
    let graphics = world.get::<Graphics>().unwrap();
    let mut meshes = world.get_mut::<Storage<Mesh>>().unwrap();
    let mut skies = world.get_mut::<Storage<SkyMaterial>>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
    let mut pipelines = world.get_mut::<Pipelines>().unwrap();

    // Get the material id (also registers the material pipeline)
    let id = pipelines
        .register::<SkyMaterial>(&graphics, &assets)
        .unwrap();

    // Create a new material instance
    let material = skies.insert(SkyMaterial {});

    // Load the renderable mesh
    let mesh = assets
        .load::<Mesh>(("engine/meshes/sphere.obj", graphics.clone()))
        .unwrap();
    let mesh = meshes.insert(mesh);

    // Create the new sky entity components
    let surface = Surface::new(mesh, material, id);
    let renderer = Renderer::default();
    scene.insert((surface, renderer));

    // Create a movable camera
    scene.insert((
        Position::default(),
        Rotation::default(),
        Velocity::default(),
        Camera {
            near: 5.0,
            ..Default::default()
        },
        ChunkViewer::default(),
        CameraController::default(),
    ));

    // Create a directional light
    let light = DirectionalLight {
        color: vek::Rgb::one() * 3.6,
    };
    let rotation = vek::Quaternion::rotation_x(-15.0f32.to_radians()).rotated_y(45f32.to_radians());
    scene.insert((light, Rotation::from(rotation)));
}

// Updates the light direction and quites from the engine
fn update(world: &mut World) {
    let time = world.get::<Time>().unwrap();
    let mut state = world.get_mut::<State>().unwrap();
    let time = &*time;
    let input = world.get::<Input>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();

    // Rotation the light
    if let Some((rotation, _)) =
        scene.find_mut::<(&mut Rotation, &DirectionalLight)>()
    {
        rotation.rotate_y(-0.1 * time.delta().as_secs_f32());
    }

    // Exit the game when the user pressed Escape
    if input.get_button(Button::Escape).pressed() {
        *state = State::Stopped;
    }
}
