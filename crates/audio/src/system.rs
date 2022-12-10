use std::sync::Arc;

use crate::{AudioPlayer, AudioSource};
use cpal::traits::DeviceTrait;
use ecs::Scene;
use parking_lot::Mutex;
use world::{post_user, user, System, World};

// Main audio update event that will play the audio clips
fn update(world: &mut World) {
    let mut scene = world.get_mut::<Scene>().unwrap();
    let listener = scene.find::<&AudioPlayer>().unwrap();

    // Iterate through all the audio sources that have been changed
    for source in scene.query_mut::<&mut AudioSource>() {
        if source.stream.is_none() && source.playing {
            let builder = source.builder().clone();
            let stream =
                builder.build_output_stream(listener).unwrap();
            source.stream = Some(stream);
        }
    }
}

// Simple audio system tbh
pub fn system(system: &mut System) {
    system.insert_update(update).after(post_user);
}
