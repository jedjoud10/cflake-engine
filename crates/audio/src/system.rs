use crate::{AudioEmitter, AudioListener};
use coords::{Position, Rotation};
use ecs::Scene;
use world::{post_user, System, World};

// Main audio update event that will play the audio clips
fn update(world: &mut World) {
    let mut scene = world.get_mut::<Scene>().unwrap();
    let listener = scene.find_mut::<(&mut AudioListener, Option<&Position>, Option<&Rotation>)>();

    // Don't do anything if we don't have an audio player
    let (listener, listener_position, listener_rotation) = match listener {
        Some((listener, Some(position), Some(rotation))) => (listener, position, rotation),
        None => {
            return;
        }
        _ => {
            log::error!("Audio listener missing Position component or Rotation component");
            return;
        }
    };

    // Update ear positions
    let left = -listener_rotation.right() * listener.ear_distance;
    let right = listener_rotation.right() * listener.ear_distance;
    *listener.ear_positions[0].write() = left + **listener_position;
    *listener.ear_positions[1].write() = right + **listener_position;

    // Iterate through all the audio sources that have been changed or added
    let filter = ecs::added::<&AudioEmitter>() | ecs::modified::<&AudioEmitter>();
    for emitter in scene.query_mut_with::<&mut AudioEmitter>(filter) {
        if emitter.stream.is_none() && emitter.playing {
            let source = emitter.source.take().unwrap();
            let stream = super::build_clip_output_stream(source, listener);
            let stream = stream.unwrap();
            cpal::traits::StreamTrait::play(&stream).unwrap();
            emitter.stream = Some(stream);
        }
    }

    // Update the positions of the positional audio emitters
    let filter = ecs::modified::<&Position>();
    for (emitter, position) in scene.query_mut_with::<(&mut AudioEmitter, &Position)>(filter) {
        if let Some(pos) = emitter.position.as_ref() {
            if let Some(mut write) = pos.try_write() {
                *write = **position;
                dbg!("write pos");
            }
        }
    }
}

// Simple audio system tbh
pub fn system(system: &mut System) {
    system.insert_update(update).after(post_user);
}
