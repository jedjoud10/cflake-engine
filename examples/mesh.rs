use cflake_engine::prelude::*;

// Mesh example game window
fn main() {
    App::default()
        .set_app_name("cflake engine mesh example")
        .set_user_assets_path(user_assets_path!("/examples/assets/"))
        .set_window_fullscreen(true)
        //.set_logging_level(LevelFilter::Trace)
        .insert_init(init)
        .insert_update(update)
        .execute();
}

// Executed at the start
fn init(world: &mut World) {
    // Create some procedural terrain
    let graphics = world.get::<Graphics>().unwrap();
    let settings = TerrainSettings::new(&graphics,
        64,
        5,
        true,
        7,
        1024,
    );
    drop(graphics);
    world.insert(settings);

    // Fetch the required resources from the world
    let mut assets = world.get_mut::<Assets>().unwrap();
    let graphics = world.get::<Graphics>().unwrap();
    let _threadpool = world.get_mut::<ThreadPool>().unwrap();
    let mut meshes = world.get_mut::<Storage<Mesh>>().unwrap();
    let mut pbrs =
        world.get_mut::<Storage<PhysicallyBasedMaterial>>().unwrap();
    let mut interface = world.get_mut::<Interface>().unwrap();
    let mut skies = world.get_mut::<Storage<SkyMaterial>>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
    let mut pipelines = world.get_mut::<Pipelines>().unwrap();

    // Make the cursor invisible and locked
    let window = world.get::<Window>().unwrap();
    window
        .raw()
        .set_cursor_grab(winit::window::CursorGrabMode::Confined)
        .unwrap();
    window.raw().set_cursor_visible(false);
    interface.enabled = false;

    // Create a terrain generator (frfr)

    // Import the diffuse map, normal map, mask map
    asset!(&mut assets, "assets/user/textures/diffuse.jpg");
    asset!(&mut assets, "assets/user/textures/normal.jpg");
    asset!(&mut assets, "assets/user/textures/mask.jpg");

    // Load in the diffuse map, normal map, and mask map textures asynchronously
    /*
    let albedo = assets.async_load::<AlbedoMap>(
        ("user/textures/diffuse.jpg", graphics.clone()),
        &mut threadpool,
    );
    let normal = assets.async_load::<NormalMap>(
        ("user/textures/normal.jpg", graphics.clone()),
        &mut threadpool,
    );
    let mask = assets.async_load::<MaskMap>(
        ("user/textures/mask.jpg", graphics.clone()),
        &mut threadpool,
    );
    */

    // Get the material id (also registers the material pipeline)
    let id = pipelines
        .register::<PhysicallyBasedMaterial>(&graphics, &mut assets)
        .unwrap();

    // Load a cube mesh
    let cube = assets
        .load::<Mesh>(("engine/meshes/cube.obj", graphics.clone()))
        .unwrap();
    let cube = meshes.insert(cube);

    // Load a plane mesh
    let plane = assets
        .load::<Mesh>(("engine/meshes/plane.obj", graphics.clone()))
        .unwrap();
    let plane = meshes.insert(plane);

    // Load a sphere mesh
    let sphere = assets
        .load::<Mesh>((
            "engine/meshes/icosphere.obj",
            graphics.clone(),
        ))
        .unwrap();
    let sphere = meshes.insert(sphere);

    /*
    // Fetch the loaded textures
    let diffuse = assets.wait(albedo).unwrap();
    let normal = assets.wait(normal).unwrap();
    let mask = assets.wait(mask).unwrap();

    // Add the textures to the storage
    let mut diffuse_maps =
        world.get_mut::<Storage<AlbedoMap>>().unwrap();
    let mut normal_maps =
        world.get_mut::<Storage<NormalMap>>().unwrap();
    let mut mask_maps = world.get_mut::<Storage<MaskMap>>().unwrap();
    let diffuse = diffuse_maps.insert(diffuse);
    let normal = normal_maps.insert(normal);
    let mask = mask_maps.insert(mask);
    */

    // Create a new material instance
    let material = pbrs.insert(PhysicallyBasedMaterial {
        albedo_map: None,
        normal_map: None,
        mask_map: None,
        bumpiness: 1.0,
        roughness: 0.1,
        metallic: 0.0,
        ambient_occlusion: 1.0,
        tint: vek::Rgb::red(),
    });

    // Create a simple floor and add the entity
    let surface =
        Surface::new(plane, material.clone(), id.clone());
    let renderer = Renderer::default();
    let scale = Scale::uniform(25.0);
    scene.insert((surface, renderer, scale));

    // Create a simple cube and add the entity
    scene.extend_from_iter((0..25).map(|x| {
        let renderer = Renderer::default();
        let position = Position::at_xyz(
            (x / 5) as f32 * 4.0,
            1.0,
            (x % 5) as f32 * 4.0,
        );

        let material = pbrs.insert(PhysicallyBasedMaterial {
            albedo_map: None,
            normal_map: None,
            mask_map: None,
            bumpiness: 4.0,
            roughness: 1.0,
            metallic: 0.2,
            ambient_occlusion: 1.0,
            tint: vek::Rgb::new(
                position.x / 5.0,
                position.z / 5.0,
                0.0,
            ),
        });

        let surface =
            Surface::new(cube.clone(), material, id.clone());
        (surface, renderer, position)
    }));

    // Create a simple sphere and add the entity
    let surface = Surface::new(sphere, material, id);
    let renderer = Renderer::default();
    let position = Position::at_y(3.5);
    scene.insert((surface, renderer, position));

    // Get the material id (also registers the material pipeline)
    let id = pipelines
        .register::<SkyMaterial>(&graphics, &mut assets)
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
        Camera::default(),
        ChunkViewer::default(),
    ));

    // Create a directional light
    let light = DirectionalLight::default();
    let rotation = vek::Quaternion::rotation_x(-25.0f32.to_radians())
        .rotated_y(45f32.to_radians());
    scene.insert((light, Rotation::from(rotation)));

    // Bind inputs to be used by the camera tick event
    let mut input = world.get_mut::<Input>().unwrap();
    input.bind_button("forward", Button::W);
    input.bind_button("backward", Button::S);
    input.bind_button("up", Button::Space);
    input.bind_button("down", Button::LControl);
    input.bind_button("left", Button::A);
    input.bind_button("right", Button::D);
    input.bind_button("lshift", Button::LShift);
    input.bind_axis("x rotation", Axis::MousePositionX);
    input.bind_axis("y rotation", Axis::MousePositionY);
}

// Camera controller update executed every tick
fn update(world: &mut World) {
    let time = world.get::<Time>().unwrap();
    let mut state = world.get_mut::<State>().unwrap();
    let time = &*time;
    let input = world.get::<Input>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();

    /*
    // Rotation the light
    if let Some((rotation, _)) =
        scene.find_mut::<(&mut Rotation, &DirectionalLight)>()
    {
        rotation.rotate_y(-0.1 * time.delta().as_secs_f32());
    }
    */

    // Exit the game when the user pressed Escape
    if input.get_button(Button::Escape).pressed() {
        *state = State::Stopped;
    }

    let camera = scene
        .find_mut::<(&mut Camera, &mut Velocity, &mut Position, &mut Rotation)>();
    if let Some((camera, output, position, rotation)) = camera {
        // Forward and right vectors relative to the camera
        let forward = rotation.forward();
        let right = rotation.right();
        let up = rotation.up();
        let mut velocity = vek::Vec3::<f32>::default();
        let mut speed: f32 = 1.0f32;
        let sensivity: f32 = 0.0007;

        // Controls the "strength" of the camera smoothness
        // Higher means more smooth, lower means less smooth
        let smoothness = 0.1;

        // Update velocity scale
        if input.get_button("lshift").held() {
            speed = 3.0f32;
        }

        // Update the velocity in the forward and backward directions
        if input.get_button("forward").held() {
            velocity += forward;
        } else if input.get_button("backward").held() {
            velocity += -forward;
        }

        // Update the velocity in the left and right directions
        if input.get_button("left").held() {
            velocity += -right;
        } else if input.get_button("right").held() {
            velocity += right;
        }

        // Update the velocity in the left and right directions
        if input.get_button("up").held() {
            velocity += up;
        } else if input.get_button("down").held() {
            velocity += -up;
        }

        // Finally multiply velocity by desired speed
        velocity *= speed;

        // Smooth velocity calculation
        let factor = (time.delta().as_secs_f32() * (1.0 / smoothness)).clamped01();
        **output = vek::Vec3::lerp(**output, velocity, (factor * 2.0).clamped01());

        // The scroll wheel will change the camera FOV
        let delta = input.get_axis(Axis::MouseScrollDelta);
        camera.hfov += delta * 10.0 * time.delta().as_secs_f32();

        // Update the position with the new velocity
        **position += **output * time.delta().as_secs_f32() * 20.0;

        // Calculate a new rotation and apply it
        let pos_x = input.get_axis("x rotation");
        let pos_y = input.get_axis("y rotation");

        // Smooth rotation
        **rotation = vek::Quaternion::lerp(**rotation, 
            vek::Quaternion::rotation_y(-pos_x * sensivity) *
            vek::Quaternion::rotation_x(-pos_y * sensivity),
            (factor * 5.0).clamped01()
        );
    }
}
