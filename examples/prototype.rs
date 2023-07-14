fn main() {}

/*
use cflake_engine::prelude::*;

// Prototype example game window
fn main() {
    App::default()
        .set_app_name("cflake engine prototype example")
        .insert_init(init)
        .set_window_fullscreen(true)
        .set_frame_rate_limit(FrameRateLimit::Limited(60))
        .insert_update(update)
        .execute();
}
// Creates a movable camera, and sky entity
fn init(world: &mut World) {
    let mut scene = world.get_mut::<Scene>().unwrap();

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

// Camera controller update executed every tick
fn update(world: &mut World) {
    let gui = world.get_mut::<Interface>().unwrap();

    egui::Window::new("Prototyping").show(&gui, |_ui| {});
}
*/