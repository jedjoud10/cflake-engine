
use cflake_engine::prelude::*;

// Player example game window
fn main() {
    App::default()
        .set_app_name("cflake engine player example")
        .insert_init(init)
        .insert_update(update)
        .set_window_fullscreen(true)
        .execute();
}

// Creates a movable player
fn init(world: &mut World) {
    // Fetch the required resources from the world
    let assets = world.get::<Assets>().unwrap();
    let graphics = world.get::<Graphics>().unwrap();
    let mut pbrs = world.get_mut::<Storage<PbrMaterial>>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
    let pipelines = world.get::<Pipelines>().unwrap();
    let mut input = world.get_mut::<Input>().unwrap();

    // Button mappings for the player controller
    input.bind_button("forward", KeyboardButton::W);
    input.bind_button("backward", KeyboardButton::S);
    input.bind_button("left", KeyboardButton::A);
    input.bind_button("right", KeyboardButton::D);

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
        albedo_map: Some(diffuse),
        normal_map: Some(normal),
        mask_map: Some(mask),
        bumpiness_factor: 0.9,
        roughness_factor: 1.0,
        metallic_factor: 1.0,
        ambient_occlusion_factor: 1.0,
        tint: vek::Rgb::white(),
        scale: vek::Extent2::one() * 25.0,
    });

    // Create a simple floor and add the entity
    let surface = Surface::new(plane, ground.clone(), id.clone());
    let renderer = Renderer::default();
    let scale = Scale::uniform(50.0);
    let rigidbody = RigidBody::new(RigidBodyType::Fixed, true);
    let collider = CuboidCollider::new(vek::Extent3::new(50.0, 0.1, 50.0), 1.0, false, None);
    scene.insert((surface, renderer, scale, rigidbody, collider));

    // Player renderer
    let surface = Surface::new(cube, ground.clone(), id.clone());
    let renderer = Renderer::default();

    // Create a camera entity
    let child = scene.insert((
        LocalPosition::at_y(1.70),
        Position::default(),
        Rotation::default(),
        LocalRotation::default(),
        Camera::default(),
        surface, renderer
    ));

    // Create a player entity
    let parent = scene.insert((
        Position::at_y(20.0),
        Velocity::default(),
        AngularVelocity::default(),
        CharacterController::new(0.02),
        CapsuleCollider::new(0.5, 1.7, 10.0, false, None),
        RigidBody::new(RigidBodyType::KinematicVelocityBased, false),
        Rotation::default(),
    ));

    // Attach the camera to the player
    scene.attach(child, parent).unwrap();

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

    // Get the player character controller
    let cr = scene.find_mut::<&mut CharacterController>().unwrap();
    let mut velocity = vek::Vec3::zero();
    
    // Update the velocity in the forward and backward directions
    if input.get_button("forward").held() {
        velocity += vek::Vec3::unit_z();
    } else if input.get_button("backward").held() {
        velocity += -vek::Vec3::unit_z();
    }

    // Update the velocity in the left and right directions
    if input.get_button("left").held() {
        velocity += -vek::Vec3::unit_x();
    } else if input.get_button("right").held() {
        velocity += vek::Vec3::unit_x();
    }

    cr.set_desired_translation(velocity * 100.0);
}
