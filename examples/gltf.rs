use cflake_engine::prelude::*;

// glTF example game window
fn main() {
    App::default()
        .set_app_name("cflake engine glTF example")
        .insert_init(init)
        .insert_update(update)
        .execute();
}

// Creates a movable camera, and sky entity
fn init(world: &mut World) {
    // Fetch the required resources from the world
    let assets = world.get::<Assets>().unwrap();

    // Setup the assets that will be loaded
    asset!(assets, "user/scenes/untitled2.glb", "/examples/assets/");

    // Load the glTF scene into the world LMAO!!
    let context = GtlfContext::from_world(world).unwrap();
    let settings = GltfSettings::default();
    assets
        .load::<GltfScene>(("user/scenes/untitled2.glb", settings, context))
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
    let light = DirectionalLight { intensity: 1.0, color: vek::Rgb::broadcast(255)  };
    let rotation = vek::Quaternion::rotation_x(-40.0f32.to_radians()).rotated_y(45f32.to_radians());
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
    if let Some((rotation, light)) = scene.find_mut::<(&mut Rotation, &mut DirectionalLight)>() {
        let value = (time.elapsed().as_secs_f32() * 0.03).sin();
        **rotation = Quaternion::rotation_x((value * 90.0 - 90.0).to_radians());
        let noon = vek::Rgb::new(255.0f32, 231.0, 204.0);
        let sunrise = vek::Rgb::new(255.0f32, 151.0, 33.0);
        let interpolated = vek::Lerp::lerp(noon, sunrise, value.abs());
        light.color = interpolated.map(|x| x as u8);
    }
}
