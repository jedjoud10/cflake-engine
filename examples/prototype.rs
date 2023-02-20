use cflake_engine::prelude::*;

// Prototype example game window
fn main() {
    App::default()
        .set_app_name("cflake engine prototype example")
        .set_user_assets_path(user_assets_path!("/examples/assets/"))
        .insert_init(init)
        .insert_update(update)
        .set_frame_rate_limit(FrameRateLimit::Unlimited)
        .set_window_fullscreen(true)
        .execute();
}

// Executed at the start
fn init(world: &mut World) {
    let mut assets = world.get_mut::<Assets>().unwrap();
    let graphics = world.get::<Graphics>().unwrap();
    let mut pipelines = world.get_mut::<Pipelines>().unwrap();
    let window = world.get::<Window>().unwrap();
    window
        .window()
        .set_cursor_grab(winit::window::CursorGrabMode::Confined)
        .unwrap();
    window.window().set_cursor_visible(false);
    pipelines.register::<Basic>(&graphics, &assets).unwrap();

    asset!(&mut assets, "assets/user/ignored/diffuse.jpg");
    asset!(&mut assets, "assets/user/ignored/normal.jpg");

    let mut textures = world.get_mut::<Storage<AlbedoMap>>().unwrap();
    let texture = assets
        .load::<AlbedoMap>((
            "user/ignored/diffuse.jpg",
            &*graphics,
        ))
    .unwrap();
    let diffuse = textures.insert(texture);

    let texture = assets
        .load::<NormalMap>((
            "user/ignored/normal.jpg",
            &*graphics,
        ))
    .unwrap();
    let normal = textures.insert(texture);

    let mut meshes = world.get_mut::<Storage<Mesh>>().unwrap();
    let mut materials = world.get_mut::<Storage<Basic>>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
    let id = pipelines.get::<Basic>().unwrap();
    
    let material = materials.insert(Basic {
        albedo_map: Some(diffuse),
        normal_map: Some(normal),
        bumpiness: 0.0,
        tint: vek::Rgb::default(),
    });

    let settings = MeshImportSettings {
        invert_triangle_ordering: false,
        ..Default::default()
    };

    let mesh = assets
        .load::<Mesh>((
            "engine/meshes/cube.obj",
            &*graphics,
            settings,
        ))
        .unwrap();
    let vertices = mesh.vertices();
    let positions =
        vertices.attribute::<attributes::Position>().unwrap();
    let mesh = meshes.insert(mesh);

    let surface = Surface::new(mesh, material, id);
    let renderer = Renderer::new(true, vek::Mat4::default());
    scene.insert((surface, renderer));

    let camera = Camera::new(120.0, 0.01, 500.0, 16.0 / 9.0);
    scene.insert((Position::default(), Rotation::default(), camera));

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

fn update(world: &mut World) {
    world.entry::<vek::Vec3<f32>>().or_default();
    let mut velocity1 = world.get_mut::<vek::Vec3<f32>>().unwrap();
    let time = world.get::<Time>().unwrap();
    let input = world.get::<Input>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();

    if input.get_button(Button::F5).pressed() {
        dbg!(time.average_fps());
    }

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

        *velocity1 = vek::Vec3::lerp(*velocity1, velocity, 0.1);

        // Update the position with the new velocity
        **position += *velocity1 * time.delta().as_secs_f32() * 20.0;

        // Calculate a new rotation and apply it
        let pos_x = input.get_axis("x rotation");
        let pos_y = input.get_axis("y rotation");
        **rotation =
            vek::Quaternion::rotation_y(-pos_x as f32 * 0.0008)
                * vek::Quaternion::rotation_x(-pos_y as f32 * 0.0008);
    }
}
