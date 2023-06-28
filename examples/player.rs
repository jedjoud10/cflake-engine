
use cflake_engine::prelude::*;

// Player example game window
fn main() {
    App::default()
        .set_app_name("cflake engine player example")
        .insert_init(init)
        .insert_update(update)
        .insert_tick(tick)
        .set_window_fullscreen(true)
        .execute();
}

// Keeps track of state between update and tick events
#[derive(Default)]
struct PlayerInputs {
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
    jump: bool,
    rotation_x: f32,
    rotation_y: f32,
}

// Creates a movable player
fn init(world: &mut World) {
    // Fetch the required resources from the world
    world.insert(PlayerInputs::default());
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
    input.bind_button("jump", KeyboardButton::Space);
    input.bind_axis("x rotation", MouseAxis::PositionX);
    input.bind_axis("y rotation", MouseAxis::PositionY);

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

    // Create a new material instance for the gound
    let material = pbrs.insert(PbrMaterial {
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
    let surface = Surface::new(plane, material.clone(), id.clone());
    let renderer = Renderer::default();
    let scale = Scale::uniform(50.0);
    let rigidbody = RigidBodyBuilder::new(RigidBodyType::Fixed).build();
    let collider = CuboidColliderBuilder::new(1.0, vek::Extent3::new(50.0, 0.1, 50.0)).build();
    scene.insert((surface, renderer, scale, rigidbody, collider));

    let surface = Surface::new(cube.clone(), material.clone(), id.clone());
    let renderer = Renderer::default();
    let rigidbody = RigidBodyBuilder::new(RigidBodyType::Fixed).build();
    let collider = CuboidColliderBuilder::new(1.0, vek::Extent3::broadcast(1.0)).build();
    scene.insert((surface, renderer, rigidbody, collider));

    // Player renderer
    let surface = Surface::new(cube.clone(), material.clone(), id.clone());
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
        CharacterController {
            max_speed: 15.0,
            acceleration: 30.0,
            direction: Default::default(),
            air_control: 0.7,
            ground_control: 1.0,
            jump_force: 12.0,
            grounded: false,
            jumping: false,
        },
        CapsuleColliderBuilder::new(2.0, 0.5, 1.7).build(),
        RigidBodyBuilder::new(RigidBodyType::Dynamic).set_locked_axes(LockedAxes::ROTATION_LOCKED).build(),
        Rotation::default(),
    ));

    // Attach the camera to the player
    scene.attach(child, parent).unwrap();

    // Create a prefab that contains the sphere entity and it's components
    let renderer = Renderer::default();
    let position = Position::default();
    let rotation = Rotation::default();
    let surface = Surface::new(sphere, material.clone(), id.clone());
    let rigidbody = RigidBodyBuilder::new(RigidBodyType::Fixed).build();
    let velocity = Velocity::default();
    let angular_velocity = AngularVelocity::default();
    let collider = SphereColliderBuilder::new(10.0, 1.0).build();
    scene.prefabify("sphere", (renderer, position, rotation, surface, rigidbody, collider, velocity, angular_velocity));

    // Create a prefab that contains the cube entity and it's components
    let renderer = Renderer::default();
    let position = Position::default();
    let rotation = Rotation::default();
    let surface = Surface::new(cube, material, id);
    let rigidbody = RigidBodyBuilder::new(RigidBodyType::Fixed).build();
    let velocity = Velocity::default();
    let angular_velocity = AngularVelocity::default();
    let collider = CuboidColliderBuilder::new(10.0, vek::Extent3::broadcast(1.0)).build();
    scene.prefabify("cube", (renderer, position, rotation, surface, rigidbody, collider, velocity, angular_velocity));

    // Create a directional light
    let light = DirectionalLight::default();
    let rotation = vek::Quaternion::rotation_x(-45.0f32.to_radians()).rotated_y(45f32.to_radians());
    scene.insert((light, Rotation::from(rotation)));

}

// Update the PlayerInputs resource
fn update(world: &mut World) {
    // Confine the user's mouse
    let mut ui = world.get_mut::<Interface>().unwrap();
    let window = world.get::<Window>().unwrap();
    window
        .raw()
        .set_cursor_grab(winit::window::CursorGrabMode::Confined)
        .unwrap();
    window.raw().set_cursor_visible(false);
    ui.consumes_window_events = true;

    // Fetch the user input state
    let mut state = world.get_mut::<State>().unwrap();
    let input = world.get::<Input>().unwrap();
    let mut player = world.get_mut::<PlayerInputs>().unwrap();

    // Exit the game when the user pressed Escape
    if input.get_button(KeyboardButton::Escape).pressed() {
        *state = State::Stopped;
    }
    
    // Update the velocity in the forward and backward directions
    player.forward = input.get_button("forward").held();
    player.backward = input.get_button("backward").held();
    player.left = input.get_button("left").held();
    player.right = input.get_button("right").held();
    player.rotation_x = input.get_axis("x rotation");
    player.rotation_y = input.get_axis("y rotation");
    player.jump |= input.get_button("jump").pressed();

    let mut scene = world.get_mut::<Scene>().unwrap();
    let (_, position, rotation) = scene.find::<(&Camera, &Position, &Rotation)>().unwrap();
    let position = rotation.forward() * 10.0 + **position;

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

// Set the required player controller force
fn tick(world: &mut World) {
    let mut scene = world.get_mut::<Scene>().unwrap();
    let mut inputs = world.get_mut::<PlayerInputs>().unwrap();

    // Local velocity
    let mut velocity = vek::Vec3::<f32>::zero();
    
    // Move the character forward and backward
    velocity.z = if inputs.forward {
        -1.0
    } else if inputs.backward {
        1.0
    } else {
        0.0
    };

    // Move the character left and right
    velocity.x = if inputs.left {
        -1.0
    } else if inputs.right {
        1.0
    } else {
        0.0
    };

    // Set the player rotation
    if let Some((_, rotation)) = scene.find_mut::<(&CharacterController, &mut Rotation)>() {
        **rotation = Quaternion::rotation_y(-inputs.rotation_x * 0.001);
    };

    // Set the camera rotation
    if let Some((_, rotation)) = scene.find_mut::<(&Camera, &mut LocalRotation)>() {
        **rotation = Quaternion::rotation_x(-inputs.rotation_y * 0.001);
    };

    // Set the global player velocity
    let (cc, rotation) = scene.find_mut::<(&mut CharacterController, &Rotation)>().unwrap();

    // Handle jumping
    if std::mem::take(&mut inputs.jump) {
        cc.jump();
    }

    // Handle movement
    if let Some(vel) = (vek::Mat4::from(rotation).mul_direction(velocity)).try_normalized() {
        cc.direction = vel;
    } else {
        cc.direction = vek::Vec3::zero();
    }

}
