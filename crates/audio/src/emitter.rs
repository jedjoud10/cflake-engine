use std::sync::Arc;

use crate::{Source, AudioListener};
use cpal::traits::StreamTrait;
use ecs::Component;
use parking_lot::RwLock;

// An audio emmiter is a component that produces sound
// Each audio emmiter is a CPAL stream that will be played
#[derive(Component)]
pub struct AudioEmitter {
    // Audio source we will play
    pub(crate) source: Option<Box<dyn Source>>,

    // These two fields get validated whenever we start playing the audio stream
    pub(crate) stream: Option<cpal::Stream>,

    // Potential position of the audio emitter
    pub(crate) position: Option<Arc<RwLock<vek::Vec3<f32>>>>,

    // Is the audio stream currently playing?
    pub(crate) playing: bool,
}

impl AudioEmitter {
    // Create a new audio emitter to play, and automatically play it on start
    pub fn new(source: impl Source + 'static) -> Self {
        Self {
            source: Some(Box::new(source)),
            stream: None,
            playing: true,
            position: None,
        }
    }

    // Create a new positional audio emitter with a position
    pub fn positional(listener: &AudioListener, source: impl Source + 'static) -> Self {
        let position: Arc<RwLock<vek::Vec3<f32>>> = Arc::default();
        
        Self {
            source: Some(Box::new(source.positional(listener.ear_positions.clone(), position.clone()))),
            stream: None,
            playing: true,
            position: Some(position),
        }
    }

    // Check if the audio emitter is currently playing
    pub fn is_playing(&self) -> bool {
        self.stream.is_some() && self.playing
    }

    // Toggles the play/resume state of the audio emitter
    pub fn toggle(&mut self) {
        if self.playing {
            self.pause()
        } else {
            self.resume();
        }
    }

    // Pause the audio emitter. No-op if it's already paused
    pub fn pause(&mut self) {
        if self.playing {
            self.playing = false;
            if let Some(stream) = &self.stream {
                stream.pause().unwrap();
            }
        }
    }

    // Resume the audio emitter. No-op if it's already playing
    pub fn resume(&mut self) {
        if !self.playing {
            self.playing = true;
            if let Some(stream) = &self.stream {
                stream.play().unwrap();
            }
        }
    }
}
