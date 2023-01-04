pub use cpal::Sample as CpalSample;
use cpal::SampleFormat;

// My own implementation of the cpal::Sample trait that is a bit more restrictive
pub trait Sample: 'static + Send + Sync + Clone + CpalSample {
    // Mix two samples together
    fn mix(self, other: Self, mix: f32) -> Self;

    // Get the cpal format of this sample
    fn format() -> SampleFormat;

    // Amplify this sample
    fn amplify(self, factor: f32) -> Self;

    // Get a silence sample
    fn zero() -> Self;

    // Create an Vec of Samples from i16
    fn from_i16_vec(vec: Vec<i16>) -> Vec<Self>;

    // Create an Vec of Samples from f32
    fn from_f32_vec(vec: Vec<f32>) -> Vec<Self>;
}

impl Sample for i16 {
    fn mix(self, other: Self, mix: f32) -> Self {
        (self.to_f32() * mix + (1.0f32 - mix) * other.to_f32())
            .to_i16()
    }

    fn format() -> SampleFormat {
        SampleFormat::I16
    }

    fn amplify(self, factor: f32) -> Self {
        let out = (self as f32 / i16::MAX as f32) * factor;
        ((out * factor) * i16::MAX as f32) as i16
    }

    fn zero() -> Self {
        0
    }

    fn from_i16_vec(vec: Vec<i16>) -> Vec<Self> {
        vec
    }

    fn from_f32_vec(vec: Vec<f32>) -> Vec<Self> {
        vec.into_iter().map(|s| s.to_i16()).collect::<_>()
    }
}

impl Sample for f32 {
    fn mix(self, other: Self, mix: f32) -> Self {
        self * mix + (1.0f32 - mix) * other
    }

    fn format() -> SampleFormat {
        SampleFormat::I16
    }

    fn amplify(self, factor: f32) -> Self {
        self * factor
    }

    fn zero() -> Self {
        0.0f32
    }

    fn from_i16_vec(vec: Vec<i16>) -> Vec<Self> {
        vec.into_iter().map(|s| s.to_f32()).collect::<_>()
    }

    fn from_f32_vec(vec: Vec<f32>) -> Vec<Self> {
        vec
    }
}
