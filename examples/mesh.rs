use cflake_engine::prelude::*;
use cflake_engine::assets::include_dir;

// Mesh example game window
fn main() {
    App::default()
        .set_app_name("cflake engine mesh example")
        .set_user_assets(user_assets!("/examples/assets/"))
        .set_window_fullscreen(true)
        .insert_init(init)
        .insert_update(update)
        .execute();
}

// Creates a movable camera, and sky entity
fn init(world: &mut World) {
    // Fetch the required resources from the world
    let mut assets = world.get_mut::<Assets>().unwrap();
    let graphics = world.get::<Graphics>().unwrap();
    let mut threadpool = world.get_mut::<ThreadPool>().unwrap();
    let mut meshes = world.get_mut::<Storage<Mesh>>().unwrap();
    let mut pbrs =
        world.get_mut::<Storage<PhysicallyBasedMaterial>>().unwrap();
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

    // Load in the diffuse map, normal map, and mask map textures asynchronously
    let albedo = assets.async_load::<AlbedoMap>(
        ("user/textures/diffuse2.jpg", graphics.clone()),
        &mut threadpool,
    );
    let normal = assets.async_load::<NormalMap>(
        ("user/textures/normal2.jpg", graphics.clone()),
        &mut threadpool,
    );
    let mask = assets.async_load::<MaskMap>(
        ("user/textures/mask2.jpg", graphics.clone()),
        &mut threadpool,
    );

    // Get the material id (also registers the material pipeline)
    let id = pipelines
        .register::<PhysicallyBasedMaterial>(&graphics, &mut assets)
        .unwrap();

    // Load a plane mesh
    let plane = assets
        .load::<Mesh>(("engine/meshes/plane.obj", graphics.clone()))
        .unwrap();
    let plane = meshes.insert(plane);

    // Load a sphere mesh
    let sphere = assets
        .load::<Mesh>((
            "engine/meshes/sphere.obj",
            graphics.clone(),
        ))
        .unwrap();
    let sphere = meshes.insert(sphere);

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

    // Create a new material instance
    let material = pbrs.insert(PhysicallyBasedMaterial {
        albedo_map: Some(diffuse),
        normal_map: Some(normal),
        mask_map: Some(mask),
        bumpiness: 1.0,
        roughness: 1.0,
        metallic: 1.0,
        ambient_occlusion: 1.0,
        tint: vek::Rgb::white(),
    });

    // Create a simple floor and add the entity
    let surface =
        Surface::new(plane, material.clone(), id.clone());
    let renderer = Renderer::default();
    let scale = Scale::uniform(25.0);
    scene.insert((surface, renderer, scale));

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
    ));

    // Create a directional light
    let light = DirectionalLight { color: vek::Rgb::one() * 3.6 };
    let rotation = vek::Quaternion::rotation_x(-15.0f32.to_radians())
        .rotated_y(45f32.to_radians());
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
