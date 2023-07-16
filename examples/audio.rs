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

    // Create an audio listener
    let mut player = AudioListener::new().unwrap();
    player.set_volume(0.010);
    scene.insert(player);

    asset!(assets, "user/audio/bruh.wav", "/examples/assets/");
    asset!(assets, "user/audio/nicolas.mp3", "/examples/assets/");

    // Load the clips from their relative paths
    let clip1 = assets.load::<AudioClip>("user/audio/bruh.wav").unwrap();
    let clip2 = assets.load::<AudioClip>("user/audio/nicolas.mp3").unwrap();

    // Play both clips at the same time
    //scene.insert(AudioEmitter::new(clip1));
    //scene.insert(AudioEmitter::new(clip2));

    // Create a square wave
    let square = Square::new(140.0, 0.5);

    // Amplify the audio source
    let square = square.amplify(0.4);

    // Apply a fade in easing effect
    let square = square.fade(
        Easing::Cosine,
        EasingDirection::In,
        std::time::Duration::from_secs_f32(6.0),
    );

    // Create a sine wave
    let sine = Sine::new(220.0);

    // Create a source that mixes them both
    let source = square.mix(sine, 0.4);

    // Add the audio emitter into the world
    let emitter = AudioEmitter::new(source);
    scene.insert(emitter);
}
