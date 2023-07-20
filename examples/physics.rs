use cflake_engine::prelude::*;

// Physics example game window
fn main() {
    App::default()
        .set_app_name("cflake engine physics example")
        .insert_init(init)
        .insert_update(update)
        .set_window_fullscreen(true)
        .set_tick_rate(64)
        .execute();
}
// Creates a movable camera, and sky entity
fn init(world: &mut World) {
    // Fetch the required resources from the world
    let assets = world.get::<Assets>().unwrap();

    // Load the glTF scene into the world LMAO!!
    let context = GtlfContext::from_world(world).unwrap();
    let settings = GltfSettings::default();
    assets
        .load::<GltfScene>(("engine/meshes/froggo.glb", settings, context))
        .unwrap();

    // Get the entity that contains the frogg and convert it to a prefab
    // TODO: Implement name search feature
    let mut scene = world.get_mut::<Scene>().unwrap();
    let froggo_surface = scene.find::<&Surface<PbrMaterial>>().unwrap();

    // Create a new material instance for the gound
    let mut pbrs = world.get_mut::<Storage<PbrMaterial>>().unwrap();
    let renderer = world.get::<DeferredRenderer>().unwrap();
    let pipelines = world.get::<Pipelines>().unwrap();
    let id = pipelines.get::<PbrMaterial>().unwrap();
    let plane = renderer.plane.clone();
    let ground = pbrs.insert(PbrMaterial {
        albedo_map: None,
        normal_map: None,
        mask_map: None,
        bumpiness_factor: 0.5,
        roughness_factor: 1.0,
        metallic_factor: 0.0,
        ambient_occlusion_factor: 1.0,
        tint: vek::Rgb::white(),
        scale: vek::Extent2::one() * 25.0,
    });

    // Create a simple floor and add the entity
    let surface = Surface::new(plane, ground.clone(), id.clone());
    let renderer = Renderer::default();
    let scale = Scale::uniform(50.0);
    let rigidbody = RigidBodyBuilder::new(RigidBodyType::Fixed).build();
    let collider = CuboidColliderBuilder::new(1.0, vek::Extent3::new(50.0, 0.03, 50.0)).build();
    scene.insert((surface, renderer, scale, rigidbody, collider));

    // Create 4 invisble collision walls
    for i in 0..4 {
        let w = if i % 2 == 0 {
            50.0
        } else {
            1.0
        };

        let h = if i % 2 == 0 {
            1.0
        } else {
            50.0
        };

        // 0, w = 50, h = 1
        // 1, w = 1, h = 50
        // 2, w = 50, h = 1
        // 3, w = 1, h = 50

        let (x, y) = match i {
            0 => (1.0, 0.0),
            1 => (0.0, 1.0),
            2 => (-1.0, 0.0),
            3 => (0.0, -1.0), 
            _ => panic!(),
        };

        let rigidbody = RigidBodyBuilder::new(RigidBodyType::Fixed).build();
        let collider = CuboidColliderBuilder::new(1.0, vek::Extent3::new(h, 50.0, w)).build();
        scene.insert((Position::at_xyz(x * 50.0, 0.0, y * 50.0), scale, rigidbody, collider));
    }

    // Create a prefab that contains the froggo entity and its components
    let renderer = Renderer::default();
    let position = Position::default();
    let rotation = Rotation::default();
    let surface = froggo_surface.clone();
    let rigidbody = RigidBodyBuilder::new(RigidBodyType::Dynamic).build();
    let velocity = Velocity::default();
    let angular_velocity = AngularVelocity::default();
    let collider = SphereColliderBuilder::new(1.0, 1.0).build();
    scene.prefabify(
        "froggo",
        (
            renderer,
            position,
            rotation,
            surface,
            rigidbody,
            collider,
            velocity,
            angular_velocity,
        ),
    );

    // Create a movable camera
    scene.insert((
        Position::default(),
        Rotation::default(),
        Velocity::default(),
        AngularVelocity::default(),
        Camera::default(),
        CameraController::default(),
    ));

    // Create a directional light
    let light = DirectionalLight { intensity: 0.8, ..Default::default() };
    let rotation = vek::Quaternion::rotation_x(-15.0f32.to_radians()).rotated_y(45f32.to_radians());
    scene.insert((light, Rotation::from(rotation)));
}

// Allows the user to place a froggo when they click
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

    // Create a new froggo in front of the camera when we press the right mouse button
    if input.get_button(MouseButton::Right).pressed() {
        let mut entry = scene.instantiate("froggo").unwrap();
        **entry.get_mut::<Position>().unwrap() = position;
        **entry.get_mut::<Velocity>().unwrap() = rotation.forward() * 15.0;
    }
}
