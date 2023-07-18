use crate::{Source, Value};

// Mix two audio sources together using a control value
pub struct Mix<A: Source, B: Source, V: Value<f32>>(
    pub(crate) A,
    pub(crate) B,
    pub(crate) V::Storage,
);

impl<A: Source, B: Source, V: Value<f32>> Source for Mix<A, B, V> {
    fn cache(&mut self) {
        self.0.cache();
        self.1.cache();
        V::cache(&mut self.2);
    }

    fn sample(&mut self, input: &crate::SourceInput) -> Option<f32> {
        let a = self.0.sample(input);
        let b = self.1.sample(input);
        let control = V::fetch(&self.2).clamp(0.0, 1.0);
        a.zip(b).map(|(a, b)| a * (1.0 - control) + b * control)
    }

    fn duration(&self) -> Option<std::time::Duration> {
        self.0
            .duration()
            .zip(self.1.duration())
            .map(|(a, b)| a.min(b))
    }

    fn target_channels(&self) -> Option<u16> {
        let a = self.0.target_channels();
        let b = self.1.target_channels();

        match (a, b) {
            (None, None) => None,
            (None, Some(x)) => Some(x),
            (Some(x), None) => Some(x),
            (Some(a), Some(b)) => {
                assert_eq!(a, b, "Target channels do not match up");
                Some(a)
            }
        }
    }

    fn target_sample_rate(&self) -> Option<u32> {
        let a = self.0.target_sample_rate();
        let b = self.1.target_sample_rate();

        match (a, b) {
            (None, None) => None,
            (None, Some(x)) => Some(x),
            (Some(x), None) => Some(x),
            (Some(a), Some(b)) => {
                assert_eq!(a, b, "Target sample rate does not match up");
                Some(a)
            }
        }
    }
}
