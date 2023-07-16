use std::time::Duration;

use crate::{AudioClip, Source, SourceInput};

impl Source for AudioClip {
    fn sample(&mut self, _input: &SourceInput) -> Option<f32> {
        todo!()
    }

    fn duration(&self) -> Option<Duration> {
        Some(AudioClip::duration(self))
    }

    fn target_channels(&self) -> Option<u16> {
        Some(AudioClip::channels(self))
    }

    fn target_sample_rate(&self) -> Option<u32> {
        Some(AudioClip::sample_rate(self))
    }
}
