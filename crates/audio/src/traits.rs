use std::{ops::Range, time::Duration};
use cpal::{Stream, BuildStreamError};

use crate::{Sample, AudioPlayer};


// Audio input data passed to modifiers / generators / mixers
pub struct AudioContext {
    // Current time the audio's been playing
    pub(crate) time_since_start: Duration,
    pub(crate) clip_duration: Option<Duration>,

    // Current frame index from CPAL 'dst'
    pub(crate) frame_index: usize,
}

impl AudioContext {
    // Get the current time since stream creation
    pub fn time_since_creation(&self) -> Duration {
        self.time_since_start
    }
    
    // Get the total audio generator duration (None if infinite)
    pub fn clip_duration(&self) -> Option<Duration> {
        self.clip_duration
    }
    
    // Get the current frame index
    pub fn frame_index(&self) -> usize {
        self.frame_index
    }
}

// An audio node is anything that makes sound and that can be turned into a stream
pub trait AudioNode<T: Sample>: Sync + Send + 'static {
    fn build_output_stream(
        &self,
        listener: &AudioPlayer,
    ) -> Result<Stream, BuildStreamError> {
        todo!()
    }
}

// Audio generators create new sound by reading from a file or creating it using a wave type
pub trait AudioGenerator<T: Sample>: AudioNode<T> {}

// Audio modifiers change how audio outputs sound
pub trait AudioModifier<T: Sample>: AudioNode<T> {}

// Audio mixers simply combine 2 or more audio outputs together
pub trait AudioMixer<T: Sample>: AudioNode<T> {}