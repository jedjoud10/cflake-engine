use cflake_engine::prelude::*;

// Mesh example game window
fn main() {
    App::default()
        .set_app_name("cflake engine mesh example")
        .set_window_fullscreen(true)
        //.set_frame_rate_limit(FrameRateLimit::VSync)
        .insert_init(init)
        .insert_update(update)
        .execute();
}

// Creates a movable camera, and sky entity
fn init(world: &mut World) {
    // Fetch the required resources from the world
    let assets = world.get::<Assets>().unwrap();

    // Setup the assets that will be loaded
    asset!(assets, "user/scenes/untitled.glb", "/examples/assets/");

    // Load the glTF scene into the world LMAO!!
    let context = GtlfContext::from_world(world).unwrap();
    let settings = GltfSettings::default();
    assets
        .load::<GltfScene>(("user/scenes/untitled.glb", settings, context))
        .unwrap();

    // Create a movable camera
    let mut scene = world.get_mut::<Scene>().unwrap();
    scene.insert((
        Position::default(),
        Rotation::default(),
        Velocity::default(),
        Camera::default(),
        CameraController::default(),
    ));

    // Create a directional light
    let light = DirectionalLight {
        color: vek::Rgb::one() * 3.6,
    };
    let rotation = vek::Quaternion::rotation_x(-15.0f32.to_radians()).rotated_y(45f32.to_radians());
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
    if input.get_button(KeyboardButton::Escape).pressed() {
        *state = State::Stopped;
    }
}
