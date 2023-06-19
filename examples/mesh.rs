
use cflake_engine::prelude::*;

// Mesh example game window
fn main() {
    App::default()
        .set_app_name("cflake engine mesh example")
        .set_window_fullscreen(true)
        .insert_init(init)
        .insert_update(update)
        .execute();
}

// Creates a movable camera, and sky entity
fn init(world: &mut World) {
    // Fetch the required resources from the world
    let assets = world.get::<Assets>().unwrap();
    let graphics = world.get::<Graphics>().unwrap();
    let mut pbrs = world.get_mut::<Storage<PbrMaterial>>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
    let pipelines = world.get::<Pipelines>().unwrap();

    asset!(assets, "user/textures/diffuse2.jpg", "/examples/assets/");
    asset!(assets, "user/textures/normal2.jpg", "/examples/assets/");
    asset!(assets, "user/textures/mask2.jpg", "/examples/assets/");

    // Load in the diffuse map, normal map, and mask map textures asynchronously
    let albedo = assets.async_load::<AlbedoMap>(("user/textures/diffuse2.jpg", graphics.clone()));
    let normal = assets.async_load::<NormalMap>(("user/textures/normal2.jpg", graphics.clone()));
    let mask = assets.async_load::<MaskMap>(("user/textures/mask2.jpg", graphics.clone()));

    // Get the material id (also registers the material pipeline)
    let id = pipelines.get::<PbrMaterial>().unwrap();

    // Get the default meshes from the forward renderer
    let renderer = world.get::<DeferredRenderer>().unwrap();
    let plane = renderer.plane.clone();
    let sphere = renderer.sphere.clone();

    // Fetch the loaded textures
    let diffuse = assets.wait(albedo).unwrap();
    let normal = assets.wait(normal).unwrap();
    let mask = assets.wait(mask).unwrap();

    // Add the textures to the storage
    let mut diffuse_maps = world.get_mut::<Storage<AlbedoMap>>().unwrap();
    let mut normal_maps = world.get_mut::<Storage<NormalMap>>().unwrap();
    let mut mask_maps = world.get_mut::<Storage<MaskMap>>().unwrap();
    let diffuse = diffuse_maps.insert(diffuse);
    let normal = normal_maps.insert(normal);
    let mask = mask_maps.insert(mask);

    // Create a new material instance
    let material = pbrs.insert(PbrMaterial {
        albedo_map: Some(diffuse),
        normal_map: Some(normal),
        mask_map: Some(mask),
        bumpiness_factor: 1.0,
        roughness_factor: 1.0,
        metallic_factor: 1.0,
        ambient_occlusion_factor: 3.0,
        tint: vek::Rgb::white(),
        scale: vek::Extent2::one(),
    });

    // Create a simple floor and add the entity
    let surface = Surface::new(plane, material.clone(), id.clone());
    let renderer = Renderer::default();
    let scale = Scale::uniform(25.0);
    scene.insert((surface, renderer, scale));

    // Create a prefab that contains the renderer, customized surface, and default position
    let renderer = Renderer::default();
    let position = Position::default();
    let surface = Surface::new(sphere, material, id);
    scene.prefabify("sphere", (renderer, position, surface));

    // ADD THE ENTITIES NOW!!
    for x in 0..25 {
        let mut entry = scene.instantiate("sphere").unwrap();
        let position = entry.get_mut::<Position>().unwrap();
        *position = Position::at_xyz((x / 5) as f32 * 4.0, 1.0, (x % 5) as f32 * 4.0);
    }

    // Create a movable camera
    scene.insert((
        Position::default(),
        Rotation::default(),
        Velocity::default(),
        Camera::default(),
        CameraController::default(),
    ));

    // Create a directional light
    let light = DirectionalLight::default();
    let rotation = vek::Quaternion::rotation_x(-15.0f32.to_radians()).rotated_y(45f32.to_radians());
    scene.insert((light, Rotation::from(rotation)));
}

// Updates the light direction and quites from the engine
fn update(world: &mut World) {
    let mut state = world.get_mut::<State>().unwrap();
    let input = world.get::<Input>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();

    // Exit the game when the user pressed Escape
    if input.get_button(KeyboardButton::Escape).pressed() {
        *state = State::Stopped;
    }
}
