use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::default()
        .insert_init(init)
        .set_user_assets_path(user_assets_path!("/examples/assets/"))
        .set_window_title("Hello World!")
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
    asset!(&mut assets, "assets/user/ignored/bruh.wav");

    // Load the clips
    let clip1 = assets.load::<AudioClip>("user/ignored/nicolas.mp3").unwrap();
    let clip2 = assets.load::<AudioClip>("user/ignored/bruh.wav").unwrap();

    // Create audio sources for the clips
    let mut source1 = AudioSource::new(clip1);
    source1.filte
    //source1.set_volume(0.0f32);
    let mut source2 = AudioSource::new(clip2);
    //source2.set_volume(0.6f32);

    // Insert both audio sources
    scene.extend_from_iter([source1, source2]);
}
