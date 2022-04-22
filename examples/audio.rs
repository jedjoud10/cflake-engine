use cflake_engine::{
    assets,
    audio::AudioSource,
    defaults,
    defaults::components::{Camera, Light, Transform},
    rendering::basics::lights::LightType,
    vek, World,
};
// An example with a test camera and some sounds
fn main() {
    cflake_engine::start("cflake-examples/camera", init)
}
// Init the simple camera and load el sounds
fn init(world: &mut World) {
    // ----Start the world----
    assets::init!("/examples/assets/");
    assets::asset!("./assets/user/sounds/nicolas.mp3");
    assets::asset!("./assets/user/sounds/mewhenthe.mp3");
    assets::asset!("./assets/user/sounds/bruh.mp3");

    defaults::systems::flycam_system::system(world);

    // Create a simple camera entity
    world.ecs.insert(|_, linker| {
        linker.insert(Camera::new(90.0, 2.0, 9000.0)).unwrap();
        linker.insert(Transform::default()).unwrap();
    });

    // Create the directional light source
    world.ecs.insert(|_, linker| {
        let light = Light(LightType::new_directional(1.0, vek::Rgb::one()));
        linker.insert(light).unwrap();
        linker.insert(Transform::rotation_x(-90f32.to_radians())).unwrap();
    });

    // Play le funny sound
    let audio = assets::load::<AudioSource>("user/sounds/mewhenthe.mp3").unwrap();
    let audio2 = assets::load::<AudioSource>("user/sounds/nicolas.mp3").unwrap();
    let audio3 = assets::load::<AudioSource>("user/sounds/bruh.mp3").unwrap();

    world.audio.play_positional(&audio, vek::Vec3::unit_x() * -2.0, |s| s).unwrap();
    world.audio.play_positional(&audio2, vek::Vec3::unit_x() * 2.0, |s| s).unwrap();
    world.audio.play_positional(&audio3, vek::Vec3::default(), |s| s).unwrap();
}
