use std::marker::PhantomData;
use cpal::Sample;

use crate::traits::{AudioOutput, AudioGenerator};

// Simple sine wave generator
pub struct SineWave<T: Sample> {
    _phantom: PhantomData<T>,
    frequency: f32,
    amplitude: f32,
    phase: f32,
}

impl<T: Sample> AudioOutput<T> for SineWave<T> {}
impl<T: Sample> AudioGenerator<T> for SineWave<T> {}

// Simple square wave generator
pub struct SquareWave<T: Sample> {
    _phantom: PhantomData<T>,
    frequency: f32,
    amplitude: f32,
    phase: f32,
}