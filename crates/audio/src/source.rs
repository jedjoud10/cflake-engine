use cpal::traits::StreamTrait;
use ecs::Component;
use crate::AudioClip;

// An audio source is a component that produces sound
// Each audio source is a CPAL stream that will be played
#[derive(Component)]
pub struct AudioSource {
    pub(crate) clip: AudioClip,
    pub(crate) volume: f32,
    pub(crate) stream: Option<cpal::Stream>,
    pub(crate) playing: bool,
}

impl AudioSource {
    // Create a new audio source to play, and automatically play it on start
    pub fn new(clip: AudioClip) -> Self {
        Self {
            clip,
            volume: 1.0,
            stream: None,
            playing: true,
        }
    }

    // Get the internal clip used
    pub fn clip(&self) -> AudioClip {
        self.clip.clone()
    }

    // Check if the audio source is currently playing
    pub fn is_playing(&self) -> bool {
        self.stream.is_some() && self.playing
    }

    // Pause the audio source
    pub fn pause(&mut self) {
        self.playing = false;
        if let Some(stream) = &self.stream {
            stream.pause().unwrap();
        }
    }

    // Resume the audio source
    pub fn resume(&mut self) {
        self.playing = true;
        if let Some(stream) = &self.stream {
            stream.play().unwrap();
        }
    }

    // Set the volume of the audio source
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    // Get the volume of the audio source
    pub fn volume(&self) -> f32 {
        self.volume
    }
}
