use std::marker::PhantomData;
use crate::{traits::{AudioNode, AudioContext}, Sample};

// This will blend two audio iterators together
pub struct Blend<IN1: AudioNode, IN2: AudioNode> {
    pub(crate) input1: IN1,
    pub(crate) input2: IN2,
    pub(crate) mix: f32
}
impl<S: Sample, IN1: AudioNode<S = S>, IN2: AudioNode<S = S>> AudioNode for Blend<IN1, IN2> {
    type S = S;
}