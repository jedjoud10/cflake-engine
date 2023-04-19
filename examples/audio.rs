use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::default()
        .set_app_name("cflake engine audio example")
        .insert_init(init)
        .execute();
}

// First function that gets executed when the engine starts
fn init(world: &mut World) {
    // Get the required resources
    let assets = world.get::<Assets>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
    scene.insert(AudioPlayer::new().unwrap());

    // Load the clips from their relative paths
    let clip1 = assets
        .load::<AudioClip<i16>>("user/audio/bruh.wav")
        .unwrap();
    let clip2 = assets
        .load::<AudioClip<i16>>("user/audio/nicolas.mp3")
        .unwrap();

    // Play both clips at the same time
    scene.insert(AudioSource::new(clip1));
    scene.insert(AudioSource::new(clip2));
}
