use std::time::Duration;

use crate::{Amplify, Buffered, Easing, EasingDirection, Fade, Mix, Repeat, Value};

// Given to the sources when they execute their "sample" method
pub struct SourceInput {
    pub channel: u16,
    pub channels: u16,
    pub index: usize,
    pub given_sample_rate: u32,
    pub duration: f32,
}

// An audio source that can generate some samples
pub trait Source: Sync + Send {
    // Called during cpal callback
    fn cache(&mut self) {}

    // Basically just an iterator
    fn sample(&mut self, input: &SourceInput) -> Option<f32>;

    // Source data
    fn duration(&self) -> Option<Duration>;

    // Channels we would like to use. Might not be what we get to use at the end
    fn target_channels(&self) -> Option<u16>;

    // Sample rate we would like to use. Might not be what we get to use at the end
    fn target_sample_rate(&self) -> Option<u32>;

    // Amplification (volume) modified
    fn amplify<V: Value>(self, volume: V) -> Amplify<V, Self>
    where
        Self: Sized,
    {
        Amplify(self, V::new_storage_from(volume))
    }

    /*
    // Buffers the audio output with a specific buffer size
    fn buffered(self, buffer_size: usize) -> Buffered<Self>
    where
        Self: Sized,
    {
        todo!()
    }
    */

    // Creates a fade in/out effect
    fn fade(self, easing: Easing, direction: EasingDirection, duration: Duration) -> Fade<Self>
    where
        Self: Sized,
    {
        Fade(self, easing, direction, duration)
    }

    // Mix two audio sources together (simple addition)
    fn mix<B: Source, V: Value>(self, other: B, control: V) -> Mix<Self, B, V>
    where
        Self: Sized,
    {
        Mix(self, other, V::new_storage_from(control))
    }

    // Repeat the given source N times after it completes execution
    // No-op if the source doesn't end
    fn repeat_with_duration(self, times: usize) -> Repeat<Self>
    where
        Self: Sized,
    {
        Repeat(self, times)
    }
}
