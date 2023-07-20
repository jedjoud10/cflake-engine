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
    let mut scene = world.get_mut::<Scene>().unwrap();

    // Create an audio listener
    let mut listener = AudioListener::new(0.1).unwrap();
    listener.set_volume(0.010);

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
    let emitter = AudioEmitter::positional(&listener, source);
    scene.insert((emitter, Position::default()));
    scene.insert((listener, Position::default(), Rotation::default()));
}

// Update the position of the audio emitter
fn update(world: &mut World) {
    let time = world.get::<Time>().unwrap();
    let mut scene = world.get_mut::<Scene>().unwrap();
    let (_, position) = scene.find_mut::<(&AudioEmitter, &mut Position)>().unwrap();
    *position = Position::at_x(time.startup().elapsed().as_secs_f32().sin() * 2.0);
}
