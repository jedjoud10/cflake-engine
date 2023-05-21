use crate::{AudioPlayer, AudioSource};
use cpal::traits::DeviceTrait;
use ecs::Scene;

use world::{post_user, System, World};

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

    // Iterate through all the audio sources that have been changed or added
    let filter = ecs::added::<&AudioSource>() | ecs::modified::<&AudioSource>();
    for source in scene.query_mut_with::<&mut AudioSource>(filter) {
        if source.stream.is_none() && source.playing {
            let stream = super::build_clip_output_stream(&source.clip, player);
            let stream = stream.unwrap();
            cpal::traits::StreamTrait::play(&stream).unwrap();
            source.stream = Some(stream);
        }
    }
}

// Simple audio system tbh
pub fn system(system: &mut System) {
    system.insert_update(update).after(post_user);
}
