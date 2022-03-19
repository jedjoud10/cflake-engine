use cflake_engine::{
    assets,
    audio::{AudioSource},
    defaults::components::{self, Camera, Transform},
    ecs::entity::ComponentLinkingGroup,
    rendering::basics::lights::{LightParameters, LightType::Directional},
    vek, World,
};
// A game with a test camera
fn main() {
    cflake_engine::start("cflake-examples", "audio", init, cflake_engine::defaults::load_debugging_systems)
}
// Init the simple camera
fn init(world: &mut World) {
    // ----Start the world----
    cflake_engine::assets::init!("/examples/assets/");
    cflake_engine::assets::asset!("./assets/user/sounds/nicolas.mp3");
    cflake_engine::assets::asset!("./assets/user/sounds/mewhenthe.mp3");
    cflake_engine::assets::asset!("./assets/user/sounds/bruh.mp3");
    // Create a simple camera entity
    let mut group = ComponentLinkingGroup::default();
    group.link(Camera::new(90.0, 2.0, 9000.0)).unwrap();
    group.link(Transform::default()).unwrap();
    world.ecs.add(group).unwrap();
    // Create the directional light source
    let light = components::Light {
        light: Directional {
            params: LightParameters::default(),
        },
    };
    let light_transform = Transform {
        rotation: vek::Quaternion::<f32>::rotation_x(-90f32.to_radians()),
        ..Default::default()
    };
    // And add it to the world as an entity
    let mut group = ComponentLinkingGroup::default();
    group.link(light_transform).unwrap();
    group.link(light).unwrap();
    world.ecs.add(group).unwrap();

    // Play le funny sound
    let audio = assets::assetc::load::<AudioSource>("user/sounds/mewhenthe.mp3").unwrap();
    let audio2 = assets::assetc::load::<AudioSource>("user/sounds/nicolas.mp3").unwrap();
    let audio3 = assets::assetc::load::<AudioSource>("user/sounds/bruh.mp3").unwrap();
    
    world
        .audio
        .play_positional(&audio, vek::Vec3::unit_x() * -2.0, |s| s)
        .unwrap();
    world
        .audio
        .play_positional(&audio2, vek::Vec3::unit_x() * 2.0, |s| s)
        .unwrap();
    world
        .audio
        .play_positional(&audio3, vek::Vec3::default(), |s| s)
        .unwrap();
}
