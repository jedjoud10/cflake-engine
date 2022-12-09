use std::marker::PhantomData;
use crate::Sample;

use crate::traits::{AudioNode, AudioContext};

// Simple sine wave generator
pub struct SineWave<S: Sample> {
    _phantom: PhantomData<S>,
    pub frequency: f32,
    pub amplitude: f32,
    pub phase: f32,
}

impl<S: Sample> AudioNode for SineWave<S> {
    type S = S;
}

// Simple square wave generator
pub struct SquareWave<S: Sample> {
    _phantom: PhantomData<S>,
    pub frequency: f32,
    pub amplitude: f32,
    pub phase: f32,
}

impl<S: Sample> AudioNode for SquareWave<S> {
    type S = S;
}