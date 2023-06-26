use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use cpal::{
    traits::{DeviceTrait, HostTrait},
};
use ecs::Component;

// This is a component that will be able to playback any type of audio to a specific cpal device
// We can technically have multiple audio listenenrs in the same scene, although that would be pretty pointless
#[derive(Component)]
pub struct AudioPlayer {
    pub(crate) device: cpal::Device,
    pub(crate) host: cpal::Host,
    pub(crate) supported_output_configs: Vec<cpal::SupportedStreamConfigRange>,
    pub(crate) volume: Arc<AtomicU32>,
}

impl AudioPlayer {
    // Create an audio player that uses the default host device
    pub fn new() -> Option<Self> {
        // Fetch the CPAL device
        let host = cpal::default_host();
        let device = host.default_output_device()?;
        log::debug!(
            "Using device {:?} as the default device for the audio player",
            device.name().unwrap()
        );

        // Fetch the cpal stream config and save them in a vec
        let supported_output_configs = device.supported_output_configs().ok()?.collect::<Vec<_>>();

        // Can't have shit in Ohio
        if supported_output_configs.is_empty() {
            panic!("No supported output configs!");
        }

        for x in supported_output_configs.iter() {
            log::debug!(
                "Min sample rate: {}, max sample rate: {}",
                x.min_sample_rate().0,
                x.max_sample_rate().0
            );
        }

        Some(Self {
            host,
            device,
            volume: Arc::new(AtomicU32::new(u32::from_ne_bytes(1.0f32.to_ne_bytes()))),
            supported_output_configs,
        })
    }

    // Set the volume of the audio player as a percentage
    pub fn set_volume(&mut self, volume: f32) {
        let value = u32::from_ne_bytes(volume.to_ne_bytes());
        self.volume.store(value, Ordering::Relaxed);
    }

    // Get the volume of the audio player
    pub fn volume(&self) -> f32 {
        let value = self.volume.load(Ordering::Relaxed);
        f32::from_ne_bytes(value.to_ne_bytes())
    }
}
