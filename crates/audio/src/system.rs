use std::sync::Arc;

use cpal::traits::DeviceTrait;
use ecs::Scene;
use parking_lot::Mutex;
use world::{System, World, user, post_user};
use crate::{AudioListener, AudioSource};

// Main audio update event that will play the audio clips
fn update(world: &mut World) {
    let mut scene = world.get_mut::<Scene>().unwrap();
    let listener = scene.first::<&AudioListener>().unwrap();
    
    // Start playing all the sounds to the listener
    let device = &listener.device;
    
    // Iterate through all the audio sources that have been changed
    for source in scene.query_mut::<&mut AudioSource>() {
        if source.stream.is_none() && source.playing {
            // Get a stream config for this singular stream with the specified descriptor
            let descriptor = source.clip.0.descriptor();
            let config = &listener
                .find_audio_stream_config(descriptor.channels, descriptor.sample_rate)
                .unwrap();

            log::debug!("Starting CPAL audio stream with {} channels and sample rate of {}", descriptor.channels, descriptor.sample_rate);
            
            // Play the audio stream and save it in the source component
            let samples = source.clip.0.clone();
            let stream = samples.build_output_stream(device, config, &source.settings).unwrap();
            source.stream = Some(stream);
        }
    }
}

// Simple audio system tbh
pub fn system(system: &mut System) {
    system.insert_update(update).after(post_user);
}
