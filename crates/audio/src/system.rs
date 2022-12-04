use world::{System, World, user, post_user};

// Initialize the audio listener (if possible)
fn init(world: &mut World) {
    if let Some(listener) = crate::AudioListener::new() {
        world.insert(listener);
    } else {
        log::warn!("Could not create an audio listener. No audio resource will be inserted");
    }
}

// Main audio update event that will play the audio clips
fn update(world: &mut World) {}

// Simple audio system tbh
pub fn system(system: &mut System) {
    system.insert_init(init).before(user);
    system.insert_update(update).after(post_user);
}
