use cpal::SampleFormat;
pub use cpal::Sample as CpalSample;

// My own implementation of the cpal::Sample trait that is a bit more restrictive
pub trait Sample: 'static + Send + Sync + Clone + CpalSample {
    // Get the cpal format of this sample
    fn format() -> SampleFormat;

    // Amplify this sample
    fn amplify(self, factor: f32) -> Self;

    // Get a silence sample
    fn zero() -> Self;
}

impl Sample for i16 {
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
}

impl Sample for f32 {
    fn format() -> SampleFormat {
        SampleFormat::I16
    }

    fn amplify(self, factor: f32) -> Self {
        self * factor
    }

    fn zero() -> Self {
        0.0f32
    }
}