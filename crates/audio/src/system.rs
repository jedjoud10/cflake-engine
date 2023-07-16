use crate::{AudioEmitter, AudioListener};

use ecs::Scene;

use world::{post_user, System, World};

// Main audio update event that will play the audio clips
fn update(world: &mut World) {
    let mut scene = world.get_mut::<Scene>().unwrap();
    let player = scene.find::<&AudioListener>();

    // Don't do anything if we don't have an audio player
    let player = if player.is_none() {
        return;
    } else {
        player.unwrap()
    };

    // Iterate through all the audio sources that have been changed or added
    let filter = ecs::added::<&AudioEmitter>() | ecs::modified::<&AudioEmitter>();
    for emitter in scene.query_mut_with::<&mut AudioEmitter>(filter) {
        if emitter.stream.is_none() && emitter.playing {
            let source = emitter.source.take().unwrap();
            let stream = super::build_clip_output_stream(source, player);
            let stream = stream.unwrap();
            cpal::traits::StreamTrait::play(&stream).unwrap();
            emitter.stream = Some(stream);
        }
    }
}

// Simple audio system tbh
pub fn system(system: &mut System) {
    system.insert_update(update).after(post_user);
}
