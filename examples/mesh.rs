use cflake_engine::prelude::*;

// Mesh example game window
fn main() {
    App::default()
        .set_app_name("cflake engine mesh example")
        .set_user_assets_path(user_assets_path!("/examples/assets/"))
        .set_window_fullscreen(true)
        .insert_init(init)
        .insert_update(update)
        .execute();
}

// Executed at the start
fn init(world: &mut World) {
    // Fetch the required resources from the world
    let mut assets = world.get_mut::<Assets>().unwrap();
    let mut threadpool = world.get_mut::<ThreadPool>().unwrap();
    let mut meshes = world.get_mut::<Storage<Mesh>>().unwrap();
    let mut basics = world.get_mut::<Storage<Basic>>().unwrap();
    let mut interface = world.get_mut::<Interface>().unwrap();
    let mut skies = world.get_mut::<Storage<Sky>>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
    let graphics = world.get::<Graphics>().unwrap();
    let mut pipelines = world.get_mut::<Pipelines>().unwrap();

    // Make the cursor invisible and locked
    let window = world.get::<Window>().unwrap();
    window
        .raw()
        .set_cursor_grab(winit::window::CursorGrabMode::Confined)
        .unwrap();
    window.raw().set_cursor_visible(false);
    interface.enabled = false;

    // Import the diffuse map and normal map
    asset!(&mut assets, "assets/user/ignored/diffuse.jpg");
    asset!(&mut assets, "assets/user/ignored/normal.jpg");
    asset!(&mut assets, "assets/user/ignored/untitled.obj");

    // Load in the diffuse map and normal map textures asynchronously
    let albedo = assets.async_load::<AlbedoMap>(
        ("user/ignored/diffuse.jpg", graphics.clone()),
        &mut threadpool,
    );
    let normal = assets.async_load::<NormalMap>(
        ("user/ignored/normal.jpg", graphics.clone()),
        &mut threadpool,
    );

    // Fetch the loaded textures
    let diffuse = assets.wait(albedo).unwrap();
    let normal = assets.wait(normal).unwrap();

    // Add the textures to the storage
    let mut diffuse_maps =
        world.get_mut::<Storage<AlbedoMap>>().unwrap();
    let mut normal_maps =
        world.get_mut::<Storage<NormalMap>>().unwrap();
    let diffuse = diffuse_maps.insert(diffuse);
    let normal = normal_maps.insert(normal);

    // Get the material id (also registers the material pipeline)
    let id =
        pipelines.register::<Basic>(&graphics, &mut assets).unwrap();

    // Create a new material instance
    let material = basics.insert(Basic {
        albedo_map: Some(diffuse),
        normal_map: Some(normal),
        bumpiness: 1.4,
        tint: vek::Rgb::one(),
    });

    // Load a cube mesh
    let cube = assets
        .load::<Mesh>(("engine/meshes/cube.obj", graphics.clone()))
        .unwrap();
    let _cube = meshes.insert(cube);

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

    // Create a simple floor and add the entity
    let surface = Surface::new(plane, material.clone(), id.clone());
    let renderer = Renderer::default();
    let scale = Scale::uniform(25.0);
    scene.insert((surface, renderer, scale));

    // Create a simple cube and add the entity
    for x in 0..25 {
        let surface = Surface::new(
            sphere.clone(),
            material.clone(),
            id.clone(),
        );
        let renderer = Renderer::default();
        let position =
            Position::at_xyz((x / 5) as f32, 0.25, (x % 5) as f32);
        scene.insert((surface, renderer, position));
    }

    // Create a simple sphere and add the entity
    let surface = Surface::new(sphere, material, id);
    let renderer = Renderer::default();
    let position = Position::at_y(1.5);
    scene.insert((surface, renderer, position));

    // Get the material id (also registers the material pipeline)
    let id =
        pipelines.register::<Sky>(&graphics, &mut assets).unwrap();

    // Create a new material instance
    let material = skies.insert(Sky { gradient_map: None });

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
    let camera = Camera::new(120.0, 0.01, 5000.0, 16.0 / 9.0);
    scene.insert((
        Position::default(),
        Rotation::default(),
        Velocity::default(),
        camera,
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
    input.bind_axis("x rotation", Axis::MousePositionX);
    input.bind_axis("y rotation", Axis::MousePositionY);
}

// Camera controller update executed every tick
fn update(world: &mut World) {
    let time = world.get::<Time>().unwrap();
    let time = &*time;
    let input = world.get::<Input>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();

    // Rotation the light
    if let Some((rotation, _)) =
        scene.find_mut::<(&mut Rotation, &DirectionalLight)>()
    {
        //rotation.rotate_x(-0.1 * time.delta().as_secs_f32());
    }

    let camera = scene
        .find_mut::<(&mut Camera, &mut Position, &mut Rotation)>();
    if let Some((camera, position, rotation)) = camera {
        // Forward and right vectors relative to the camera
        let forward = rotation.forward();
        let right = rotation.right();
        let up = rotation.up();
        let mut velocity = vek::Vec3::<f32>::default();

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

        // The scroll wheel will change the camera FOV
        let delta = input.get_axis(Axis::MouseScrollDelta);
        camera.hfov += delta * 10.0 * time.delta().as_secs_f32();
        camera.update_projection();

        // Update the position with the new velocity
        **position += velocity * time.delta().as_secs_f32() * 20.0;

        // Calculate a new rotation and apply it
        let pos_x = input.get_axis("x rotation");
        let pos_y = input.get_axis("y rotation");
        **rotation = vek::Quaternion::rotation_y(-pos_x * 0.0007)
            * vek::Quaternion::rotation_x(-pos_y * 0.0007);
    }
}
