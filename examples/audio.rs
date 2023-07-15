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
    
    // Create an audio listener
    let mut player = AudioListener::new().unwrap();
    player.set_volume(0.25);
    scene.insert(player);

    asset!(assets, "user/audio/bruh.wav", "/examples/assets/");
    asset!(assets, "user/audio/nicolas.mp3", "/examples/assets/");

    // Load the clips from their relative paths
    let clip1 = assets.load::<AudioClip>("user/audio/bruh.wav").unwrap();
    let clip2 = assets.load::<AudioClip>("user/audio/nicolas.mp3").unwrap();

    // Play both clips at the same time
    //scene.insert(AudioEmitter::new(clip1));
    //scene.insert(AudioEmitter::new(clip2));

    let arc = std::sync::Arc::new(AtomicF32::new(1.0));
    let source = Sine::sine(440.0)
        .amplify(arc.clone());
    let emitter = AudioEmitter::new(source);
    scene.insert(emitter);

    drop(scene);
    drop(assets);
    world.insert(arc);
}

// Update amplification
fn update(world: &mut World) {
    let frequency = world.get::<std::sync::Arc<AtomicF32>>().unwrap();
    let time = world.get::<Time>().unwrap();
    let sin = time.startup().elapsed().as_secs_f32().sin() * 0.5 + 0.5;
    frequency.store(sin, std::sync::atomic::Ordering::Relaxed);
}
