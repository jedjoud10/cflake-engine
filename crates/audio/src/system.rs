use std::sync::Arc;

use crate::{AudioPlayer, AudioSource};
use cpal::traits::DeviceTrait;
use ecs::Scene;
use parking_lot::Mutex;
use world::{post_user, user, System, World};

// Main audio update event that will play the audio clips
fn update(world: &mut World) {
    let mut scene = world.get_mut::<Scene>().unwrap();
    let player = scene.find::<&AudioPlayer>();

    // Don't do anything if we don't have an audio player
    let player = if player.is_none() {
        return;
    } else {
        player.unwrap()
    };

    // Iterate through all the audio sources that have been changed
    for source in scene.query_mut::<&mut AudioSource>() {
        if source.stream.is_none() && source.playing {
            let stream =
                source.builder.build_output_stream(player).unwrap();
            source.stream = Some(stream);
        }
    }
}

// Simple audio system tbh
pub fn system(system: &mut System) {
    system.insert_update(update).after(post_user);
}