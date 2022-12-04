use cflake_engine::prelude::*;

// An empty game window
fn main() {
    App::default()
        .insert_init(init)
        .set_window_title("Hello World!")
        .execute();
}

// First function that gets executed when the engine starts
fn init(world: &mut World) {
    // Get the required resources
    let assets = world.get_mut::<Assets>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();

    // Load an audio file
    let audio = assets.load::<AudioClip>("test.ogg").unwrap();

    // Play the audio file
    scene.insert(AudioSource::new(audio));    
}
