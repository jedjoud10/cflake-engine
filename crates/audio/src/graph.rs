use std::{ops::Range, time::Duration, marker::PhantomData};
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

// An audio node is anything that can make sound
trait AudioNode {
    fn next(&self);
    fn volume(&self)
}