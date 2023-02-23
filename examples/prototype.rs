use cflake_engine::prelude::*;

// Prototype example game window
fn main() {
    App::default()
        .set_app_name("cflake engine prototype example")
        .set_user_assets_path(user_assets_path!("/examples/assets/"))
        .set_window_fullscreen(true)
        .insert_init(init)
        .insert_tick(tick)
        .execute();
}

// Executed at the start
fn init(world: &mut World) {
    // Fetch the required resources from the world
    let mut assets = world.get_mut::<Assets>().unwrap();
    let mut threadpool = world.get_mut::<ThreadPool>().unwrap();
    let mut meshes = world.get_mut::<Storage<Mesh>>().unwrap();
    let mut materials = world.get_mut::<Storage<Basic>>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
    let graphics = world.get::<Graphics>().unwrap();
    let mut pipelines = world.get_mut::<Pipelines>().unwrap();
    
    // Make the cursor invisible and locked
    let window = world.get::<Window>().unwrap();
    window
        .window()
        .set_cursor_grab(winit::window::CursorGrabMode::Confined)
        .unwrap();
    window.window().set_cursor_visible(false);

    // Import the diffuse map and normal map
    asset!(&mut assets, "assets/user/ignored/diffuse.jpg");
    asset!(&mut assets, "assets/user/ignored/normal.jpg");

    // Load in the diffuse map and normal map textures asynchronously
    let handles = assets
        .async_load_from_iter::<AlbedoMap>([(
            "user/ignored/diffuse.jpg",
            graphics.clone(),
        ), (
            "user/ignored/normal.jpg",
            graphics.clone(),
        )], &mut threadpool);
    
    // Fetch the loaded textures
    let mut textures = assets.wait_from_iter(handles);
    let normal = textures.pop().unwrap().unwrap();
    let diffuse = textures.pop().unwrap().unwrap();

    // Add the textures to the storage
    let mut textures = world.get_mut::<Storage<AlbedoMap>>().unwrap();
    let diffuse = textures.insert(diffuse);
    let normal = textures.insert(normal);

    // Get the material id (also registers the material pipeline)
    let id = pipelines.register::<Basic>(&graphics, &assets).unwrap();
    
    // Create a new material instance
    let material = materials.insert(Basic {
        albedo_map: Some(diffuse),
        normal_map: Some(normal),
        bumpiness: 10.0,
        tint: vek::Rgb::one(),
    });

    // Load the renderable mesh
    let mesh = assets
        .load::<Mesh>((
            "engine/meshes/cube.obj",
            graphics.clone(),
        ))
        .unwrap();
    let mesh = meshes.insert(mesh);

    // Create the new mesh entity components
    for x in 0..10 {
        for y in 0..10 {
            let surface = Surface::new(mesh.clone(), material.clone(), id.clone());
            let renderer = Renderer::default();
            let position = Position::at_xyz(x as f32 * 2.0, y as f32 * 2.0, 0.0);
            scene.insert((surface, renderer, position));
        }
    }

    // Create a movable camera (through the tick event)
    let camera = Camera::new(120.0, 0.01, 500.0, 16.0 / 9.0);
    scene.insert((Position::default(), Rotation::default(), camera));

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
fn tick(world: &mut World) {
    let time = world.get::<Time>().unwrap();
    let input = world.get::<Input>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
    let camera =
        scene.find_mut::<(&Camera, &mut Position, &mut Rotation)>();
    if let Some((_, position, rotation)) = camera {
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

        // Update the position with the new velocity
        **position += velocity * time.tick_delta().as_secs_f32() * 20.0;

        // Calculate a new rotation and apply it
        let pos_x = input.get_axis("x rotation");
        let pos_y = input.get_axis("y rotation");
        **rotation =
            vek::Quaternion::rotation_y(-pos_x as f32 * 0.0008)
                * vek::Quaternion::rotation_x(-pos_y as f32 * 0.0008);
    }
}
