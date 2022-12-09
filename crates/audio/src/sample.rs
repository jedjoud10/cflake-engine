use cpal::SampleFormat;

// My own implementation of the cpal::Sample trait that is a bit more restrictive
// Literally 100% of this implementation is stolen from cpal::Sample lol
pub trait Sample: 'static + Send + Sync + Clone {
    // Get the cpal format of this sample
    fn format() -> SampleFormat;

    // Convert to f32, i16, u16
    fn to_f32(&self) -> f32;
    fn to_i16(&self) -> i16;

    // Amplify this sample
    fn amplify(self, factor: f32) -> Self;
    fn zero() -> Self;
}

impl Sample for i16 {
    fn format() -> SampleFormat {
        SampleFormat::I16
    }

    fn to_f32(&self) -> f32 {
        if *self < 0 {
            *self as f32 / -(i16::MIN as f32)
        } else {
            *self as f32 / i16::MAX as f32
        }
    }

    fn to_i16(&self) -> i16 {
        *self
    }

    fn amplify(self, factor: f32) -> Self {
        let out = (self as f32 / i16::MAX as f32) * factor;
        ((out * factor) * i16::MAX as f32) as i16
    }

    fn zero() -> Self {
        0
    }
}

const F32_TO_16BIT_INT_MULTIPLIER: f32 = u16::MAX as f32 * 0.5;
impl Sample for f32 {
    fn format() -> SampleFormat {
        SampleFormat::I16
    }

    fn to_f32(&self) -> f32 {
        *self
    }

    fn to_i16(&self) -> i16 {
        if *self >= 0.0 {
            (*self * i16::MAX as f32) as i16
        } else {
            (-*self * i16::MIN as f32) as i16
        }
    }

    fn amplify(self, factor: f32) -> Self {
        self * factor
    }

    fn zero() -> Self {
        0.0f32
    }
}