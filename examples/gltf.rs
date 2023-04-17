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

    asset!(assets, "user/scenes/untitled.gltf");

    // Load the glTF scene into the world LMAO!!
    let context = GtlfContext::from_world(world).unwrap();
    let settings = GltfSettings::default();
    assets.load::<GltfScene>(("user/scenes/untitled.gltf", settings, context)).unwrap();

    let graphics = world.get::<Graphics>().unwrap();
    let mut meshes = world.get_mut::<Storage<Mesh>>().unwrap();
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

    // Get the material id (also registers the material pipeline)
    let id = pipelines
        .register::<SkyMaterial>(&graphics, &assets)
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
    let rotation = vek::Quaternion::rotation_x(-90.0f32.to_radians())
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
    /*
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
}
