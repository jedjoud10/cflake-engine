use crate::{Source, Value};

// Amplify a source by a specific amount
pub struct Amplify<V: Value<f32>, T: Source>(pub(crate) T, pub(crate) V::Storage);

impl<T: Source, V: Value<f32>> Source for Amplify<V, T> {
    fn cache(&mut self) {
        self.0.cache();
        V::cache(&mut self.1);
    }

    fn sample(&mut self, input: &crate::SourceInput) -> Option<f32> {
        self.0.sample(input).map(|x| x * V::fetch(&self.1))
    }

    fn duration(&self) -> Option<std::time::Duration> {
        self.0.duration()
    }

    fn target_channels(&self) -> Option<u16> {
        self.0.target_channels()
    }

    fn target_sample_rate(&self) -> Option<u32> {
        self.0.target_sample_rate()
    }
}

// Amplify a source by a specific amount for each channel
pub struct ChannelAmplify<const C: usize, V: Value<f32>, T: Source>(
    pub(crate) T,
    pub(crate) [V::Storage; C],
);

impl<const C: usize, T: Source, V: Value<f32>> Source for ChannelAmplify<C, V, T> {
    fn cache(&mut self) {
        self.0.cache();

        for i in 0..C {
            V::cache(&mut self.1[i]);
        }
    }

    fn sample(&mut self, input: &crate::SourceInput) -> Option<f32> {
        self.0
            .sample(input)
            .map(|x| x * V::fetch(&self.1[input.channel as usize]))
    }

    fn duration(&self) -> Option<std::time::Duration> {
        self.0.duration()
    }

    fn target_channels(&self) -> Option<u16> {
        Some(C as u16)
    }

    fn target_sample_rate(&self) -> Option<u32> {
        self.0.target_sample_rate()
    }
}
