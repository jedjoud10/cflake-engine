use crate::Source;
use cpal::traits::StreamTrait;
use ecs::Component;

// An audio emmiter is a component that produces sound
// Each audio emmiter is a CPAL stream that will be played
#[derive(Component)]
pub struct AudioEmitter {
    // Audio source we will play
    pub(crate) source: Option<Box<dyn Source>>,

    // These two fields get validated whenever we start playing the audio stream
    pub(crate) stream: Option<cpal::Stream>,

    // Is the audio stream currently playing?
    pub(crate) playing: bool,
}

impl AudioEmitter {
    // Create a new audio source to play, and automatically play it on start
    pub fn new(source: impl Source + 'static) -> Self {
        Self {
            source: Some(Box::new(source)),
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
