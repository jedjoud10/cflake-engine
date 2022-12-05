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
    asset!(&mut assets, "assets/user/ignored/bruh.mp3");
    let clip = assets.load::<AudioClip>("user/ignored/bruh.mp3").unwrap();
    scene.insert(AudioListener::new().unwrap());

    // Create an audio source
    let mut source = AudioSource::new(clip);
    source.set_volume(0.7);
    scene.insert(source);

    /*
    // Load an audio file
    let audio = assets.load::<AudioClip>("test.ogg").unwrap();

    // Play the audio file
    scene.insert(AudioSource::new(audio));    
    */
}
