use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use cpal::{
    traits::{DeviceTrait, HostTrait},
    StreamConfig,
};
use ecs::Component;

// This is a component that will be able to playback any type of audio to a specific cpal device
// We can technically have multiple audio listenenrs in the same scene, although that would be pretty pointless
#[derive(Component)]
pub struct AudioPlayer {
    pub(crate) device: cpal::Device,
    pub(crate) host: cpal::Host,
    pub(crate) supported_output_configs:
        Vec<cpal::SupportedStreamConfigRange>,
    volume: Arc<AtomicU32>,
}

impl AudioPlayer {
    // Create an audio player that uses the default host device
    pub fn new() -> Option<Self> {
        // Fetch the CPAL device
        let host = cpal::default_host();
        let device = host.default_output_device()?;
        log::debug!("Using device {:?} as the default device for the audio player",
            device.name().unwrap()
        );

        // Fetch the cpal stream config and save them in a vec
        let supported_output_configs = device
            .supported_output_configs()
            .ok()?
            .collect::<Vec<_>>();

        // Logging the supported output configs
        for config in supported_output_configs.iter() {
            log::debug!("{config:#?}");
        }

        Some(Self {
            host,
            device,
            volume: Arc::new(AtomicU32::new(u32::MAX)),
            supported_output_configs,
        })
    }

    // Try to find an audio stream config that supports the given sample rate and given channels
    pub fn find_audio_stream_config(
        &self,
        channels: u16,
        sample_rate: u32,
    ) -> Option<StreamConfig> {
        log::debug!("Looking for audio stream config for sample rate = {sample_rate} with {channels} channels");

        self.supported_output_configs
            .iter()
            .find(|config_range| {
                let channels = config_range.channels() == channels;
                let max = config_range.max_sample_rate().0 > sample_rate;
                let min = config_range.min_sample_rate().0 < sample_rate;
                log::debug!("Channels supported: {channels}");
                log::debug!("Max sample rate supported: {max}");
                log::debug!("Min sample rate supported: {min}");
                channels & max & min
            })
            .map(|p| {
                p.clone()
                    .with_sample_rate(cpal::SampleRate(sample_rate))
                    .config()
            })
    }

    // Set the volume of the audio player as a percentage
    // Panics if the value is higher than 1.0
    pub fn set_volume(&mut self, volume: f32) {
        assert!(volume <= 1.0);
        let value = volume / (u32::MAX as f32);
        self.volume.store(value as u32, Ordering::Relaxed);
    }

    // Get the volume of the audio player
    pub fn volume(&self) -> f32 {
        let value = self.volume.load(Ordering::Relaxed);
        value as f32 / (u32::MAX as f32)
    }
}
