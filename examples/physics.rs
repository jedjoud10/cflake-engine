
use cflake_engine::prelude::*;

// Physics example game window
fn main() {
    App::default()
        .set_app_name("cflake engine physics example")
        .insert_init(init)
        .insert_update(update)
        .set_window_fullscreen(true)
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
    let cube = renderer.cube.clone();

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

    // Create a new material instance for the gound
    let ground = pbrs.insert(PbrMaterial {
        albedo_map: Some(diffuse.clone()),
        normal_map: Some(normal.clone()),
        mask_map: Some(mask.clone()),
        bumpiness_factor: 0.9,
        roughness_factor: 1.0,
        metallic_factor: 1.0,
        ambient_occlusion_factor: 1.0,
        tint: vek::Rgb::white(),
        scale: vek::Extent2::one() * 25.0,
    });

    // Create a new material instance for the cubes and spheres
    let material = pbrs.insert(PbrMaterial {
        albedo_map: None,
        normal_map: Some(normal),
        mask_map: Some(mask),
        bumpiness_factor: 0.9,
        roughness_factor: 1.0,
        metallic_factor: 1.0,
        ambient_occlusion_factor: 1.0,
        tint: vek::Rgb::white(),
        scale: vek::Extent2::one(),
    });

    // Create a simple floor and add the entity
    let surface = Surface::new(plane, ground.clone(), id.clone());
    let renderer = Renderer::default();
    let scale = Scale::uniform(50.0);
    let rigidbody = RigidBody::new(RigidBodyType::Fixed, true, LockedAxes::empty());
    let collider = CuboidCollider::new(vek::Extent3::new(50.0, 0.03, 50.0), 1.0, false, None);
    scene.insert((surface, renderer, scale, rigidbody, collider));

    // Create a prefab that contains the sphere entity and it's components
    let renderer = Renderer::default();
    let position = Position::default();
    let rotation = Rotation::default();
    let surface = Surface::new(sphere, material.clone(), id.clone());
    let rigidbody = RigidBody::new(RigidBodyType::Dynamic, true, LockedAxes::empty());
    let velocity = Velocity::default();
    let angular_velocity = AngularVelocity::default();
    let collider = SphereCollider::new(1.0, 1.0, false, None);
    scene.prefabify("sphere", (renderer, position, rotation, surface, rigidbody, collider, velocity, angular_velocity));

    // Create a prefab that contains the cube entity and it's components
    let renderer = Renderer::default();
    let position = Position::default();
    let rotation = Rotation::default();
    let surface = Surface::new(cube, material, id);
    let rigidbody = RigidBody::new(RigidBodyType::Dynamic, true, LockedAxes::empty());
    let velocity = Velocity::default();
    let angular_velocity = AngularVelocity::default();
    let collider = CuboidCollider::new(vek::Extent3::broadcast(1.0), 10.0, false, None);
    scene.prefabify("cube", (renderer, position, rotation, surface, rigidbody, collider, velocity, angular_velocity));

    // Create a movable camera
    let collider = SphereCollider::new(2.0, 1.0, false, None);
    scene.insert((
        Position::default(),
        Rotation::default(),
        Velocity::default(),
        AngularVelocity::default(),
        Camera::default(),
        CameraController::default(),
        RigidBody::new(RigidBodyType::KinematicPositionBased, false, LockedAxes::empty()),
        collider,
    ));

    // Create a directional light
    let light = DirectionalLight::default();
    let rotation = vek::Quaternion::rotation_x(-45.0f32.to_radians()).rotated_y(45f32.to_radians());
    scene.insert((light, Rotation::from(rotation)));
}

// Allows the user to place a sphere or cube when they left or right click
fn update(world: &mut World) {
    let mut state = world.get_mut::<State>().unwrap();
    let input = world.get::<Input>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();

    // Exit the game when the user pressed Escape
    if input.get_button(KeyboardButton::Escape).pressed() {
        *state = State::Stopped;
    }

    let (_, position, rotation) = scene.find::<(&Camera, &Position, &Rotation)>().unwrap();
    let position = rotation.forward() * 3.0 + **position;

    // Create a new sphere in front of the camera when we press the right mouse button
    if input.get_button(MouseButton::Right).pressed() {
        let mut entry = scene.instantiate("sphere").unwrap();
        **entry.get_mut::<Position>().unwrap() = position;
        **entry.get_mut::<Velocity>().unwrap() = rotation.forward() * 15.0;
    }

    // Create a new box in front of the camera when we press the left mouse button
    if input.get_button(MouseButton::Left).pressed() {
        let mut entry = scene.instantiate("cube").unwrap();
        **entry.get_mut::<Position>().unwrap() = position;
        **entry.get_mut::<Velocity>().unwrap() = rotation.forward() * 15.0;
    }
}
