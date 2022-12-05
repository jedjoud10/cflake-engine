use std::sync::Arc;

use cpal::traits::StreamTrait;
use ecs::Component;
use parking_lot::Mutex;
use crate::{AudioClip, AudioSamplesSettings};

// An audio source is a component that produces sound
// Each audio source is a CPAL stream that will be played
#[derive(Component)]
pub struct AudioSource {
    // Audio clip that the user wishes to play
    pub(crate) clip: AudioClip,
    
    // Volume and effects applied to the audio samples
    pub(crate) settings: AudioSamplesSettings,

    // These two fields get validated whenever we start playing the audio stream
    pub(crate) stream: Option<cpal::Stream>,
    
    // Is the audio stream currently playing?
    pub(crate) playing: bool,
}

impl AudioSource {
    // Create a new audio source to play, and automatically play it on start
    pub fn new(clip: AudioClip) -> Self {
        Self {
            clip,
            settings: AudioSamplesSettings {
                volume: Arc::new(Mutex::new(1.0f32)),
                callback: None,
            },
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
        *self.settings.volume.lock() = volume;
    }

    // Get the volume of the audio source
    pub fn volume(&self) -> f32 {
        *self.settings.volume.lock()
    }
}
