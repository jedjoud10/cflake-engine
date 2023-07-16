use crate::Source;
use std::time::Duration;

// Repeat the given source N times
pub struct Repeat<T: Source>(pub(crate) T, pub(crate) usize);

impl<T: Source> Source for Repeat<T> {
    fn cache(&mut self) {
        self.0.cache();
    }

    fn sample(&mut self, input: &crate::SourceInput) -> Option<f32> {
        match self.0.duration() {
            Some(duration) => {
                let index = input.index as f32
                    % (input.given_sample_rate as f32
                        * duration.as_secs_f32()
                        * input.channels as f32);

                let input = crate::SourceInput {
                    channel: input.channel,
                    index: index as usize,
                    given_sample_rate: input.given_sample_rate,
                    duration: input.duration % duration.as_secs_f32(),
                    channels: input.channels,
                };

                self.0.sample(&input)
            }
            None => self.0.sample(input),
        }
    }

    fn duration(&self) -> Option<Duration> {
        match self.0.duration() {
            Some(x) => Some(Duration::from_secs_f32(x.as_secs_f32() * self.1 as f32)),
            None => None,
        }
    }

    fn target_channels(&self) -> Option<u16> {
        self.0.target_channels()
    }

    fn target_sample_rate(&self) -> Option<u32> {
        self.0.target_sample_rate()
    }
}
