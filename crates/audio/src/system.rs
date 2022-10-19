use world::{Events, Update, World, ThreadPool};
use crate::AudioListener;

// Main audio update event that will play the audio clips to the audio stream and sheize
fn update(world: &mut World) {
}


// Simple audio system tbh
pub fn system(events: &mut Events) {
    events.registry::<Update>().insert(update);
}