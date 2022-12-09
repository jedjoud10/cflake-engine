use std::{ops::Range, time::Duration, marker::PhantomData};
use cpal::{Stream, BuildStreamError};
use crate::{Sample, AudioPlayer, Volume, Blend};

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
pub trait AudioNode: Sized + Sync + Send + 'static {
    // Sample type that we are using
    type S: Sample;
    
    // Fill a buffer with the next sound
    fn next(&self, dst: &mut [Self::S], context: &AudioContext) {}

    // Apply a volume modifier on this audio node
    fn volume(self, volume: f32) -> Volume<Self> {
        Volume {
            input: self,
            volume,
        }
    }

    // Apply a positional modifier on this audio node
    
    // Blend this node with another node
    fn blend<Other: AudioNode<S = Self::S>>(self, other: Other, mix: f32) -> Blend<Self, Other> {
        Blend {
            input1: self,
            input2: other,
            mix,
        }
    }
}