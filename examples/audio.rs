use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::default()
        .set_app_name("cflake engine audio example")
        .insert_init(init)
        .insert_update(update)
        .execute();
}

// First function that gets executed when the engine starts
fn init(world: &mut World) {
    // Get the required resources
    let assets = world.get::<Assets>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
    scene.insert(AudioPlayer::new().unwrap());

    asset!(assets, "user/audio/bruh.wav", "/examples/assets/");
    asset!(assets, "user/audio/nicolas.mp3", "/examples/assets/");

    // Load the clips from their relative paths
    let clip1 = assets.load::<AudioClip>("user/audio/bruh.wav").unwrap();
    let clip2 = assets.load::<AudioClip>("user/audio/nicolas.mp3").unwrap();

    // Play both clips at the same time
    scene.insert(AudioSource::new(clip1));
    scene.insert(AudioSource::new(clip2));
}

// Changes the volume of the audio player based on sin
fn update(world: &mut World) {
    let mut scene = world.get_mut::<Scene>().unwrap();
    let mut player = scene.find_mut::<&mut AudioPlayer>().unwrap();
    let time = world.get::<Time>().unwrap();
    player.set_volume((time.elapsed().as_secs_f32().sin() + 1.0) / 2.0);
}
