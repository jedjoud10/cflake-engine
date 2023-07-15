use crate::{Source, Value};


// Amplify a source by a specific amount
pub struct Amplify<V: Value, T: Source>(pub(crate) T, pub(crate) V::Storage);

impl<T: Source, V: Value> Source for Amplify<V, T> {
    fn cache(&mut self) {
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