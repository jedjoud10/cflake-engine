use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::default()
        .insert_init(init)
        .set_user_assets_path(user_assets_path!("/examples/assets/"))
        .set_app_name("Hello World!")
        .execute();
}

// First function that gets executed when the engine starts
fn init(world: &mut World) {
    // Get the required resources
    let mut assets = world.get_mut::<Assets>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
    scene.insert(AudioPlayer::new().unwrap());

    // Register the assets
    asset!(&mut assets, "assets/user/ignored/nicolas.mp3");
    asset!(&mut assets, "assets/user/ignored/bruh.mp3");
    asset!(&mut assets, "assets/user/ignored/bruh.wav");

    // Load the clips from their relative paths
    let _clip1 = assets
        .load::<AudioClip<i16>>("user/ignored/bruh.mp3")
        .unwrap();
    let _clip2 = assets
        .load::<AudioClip<i16>>("user/ignored/bruh.wav")
        .unwrap();
    let clip3 = assets
        .load::<AudioClip<i16>>("user/ignored/nicolas.mp3")
        .unwrap();

    scene.insert(AudioSource::new(clip3));
}
