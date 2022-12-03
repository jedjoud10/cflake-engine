use cpal::traits::HostTrait;

// This is a component that will be able to playback any type of audio to a specific cpal device
// We can technically have multiple audio listenenrs in the same scene, although that would be pretty pointless
pub struct AudioListener {
    pub(crate) device: cpal::Device,
    volume: f32,
}

impl AudioListener {
    // Create an audio listener that uses the default host device
    pub fn new() -> Option<Self> {
        let host = cpal::default_host();
        let device = host.default_output_device()?;

        Some(Self {
            volume: 1.0,
            device,
        })
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
