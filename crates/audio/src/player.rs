use cpal::{traits::{HostTrait, DeviceTrait}, StreamConfig};
use ecs::Component;

// This is a component that will be able to playback any type of audio to a specific cpal device
// We can technically have multiple audio listenenrs in the same scene, although that would be pretty pointless
#[derive(Component)]
pub struct AudioPlayer {
    pub(crate) device: cpal::Device,
    pub(crate) host: cpal::Host,
    pub(crate) supported_output_configs: Vec<cpal::SupportedStreamConfigRange>,
    volume: f32,
}

impl AudioPlayer {
    // Create an audio listener that uses the default host device
    pub fn new() -> Option<Self> {
        // Fetch the CPAL device
        let host = cpal::default_host();
        let device = host.default_output_device()?;        
        log::debug!("Using device {:?} as the default device for the audio listener",
            device.name().unwrap()
        );

        // Fetch the cpal stream config and save them in a vec
        let supported_output_configs = device
            .supported_output_configs()
            .ok()?
            .collect::<Vec<_>>(); 

        Some(Self {
            host,
            device,
            volume: 1.0,
            supported_output_configs,
        })
    }

    // Try to find an audio stream config that supports the given sample rate and given channels
    pub fn find_audio_stream_config(&self, channels: u16, sample_rate: u32,) -> Option<StreamConfig> {
        self.supported_output_configs.iter().find(|config_range| 
                config_range.channels() == channels
                && config_range.max_sample_rate().0 > sample_rate
                && config_range.min_sample_rate().0 < sample_rate
            ).map(|p| p.clone().with_sample_rate(cpal::SampleRate(sample_rate)).config())
    }

    // Set the volume of the audio listener
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume
    }
    
    // Get the volume of the audio listener
    pub fn volume(&self) -> f32 {
        self.volume
    }
}
