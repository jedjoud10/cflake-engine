use std::sync::Arc;
use cpal::traits::StreamTrait;
use ecs::Component;

use crate::AudioClip;

// An audio source is a component that produces sound
// Each audio source is a CPAL stream that will be played
#[derive(Component)]
pub struct AudioSource {
    // Audio clipthat we will play
    pub(crate) clip: AudioClip,

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
            stream: None,
            playing: true,
        }
    }

    // Check if the audio source is currently playing
    pub fn is_playing(&self) -> bool {
        self.stream.is_some() && self.playing
    }

    // Toggles the play/resume state of the audio source
    pub fn toggle(&mut self) {
        if self.playing {
            self.pause()
        } else {
            self.resume();
        }
    }

    // Pause the audio source. No-op if it's already paused
    pub fn pause(&mut self) {
        if self.playing {
            self.playing = false;
            if let Some(stream) = &self.stream {
                stream.pause().unwrap();
            }
        }
    }

    // Resume the audio source. No-op if it's already playing
    pub fn resume(&mut self) {
        if !self.playing {
            self.playing = true;
            if let Some(stream) = &self.stream {
                stream.play().unwrap();
            }
        }
    }
}
