use std::marker::PhantomData;
use crate::{traits::{AudioNode, AudioMixer, AudioModifier}, Sample};

// An audio graph is a way for us to generate, modify, and mix multiple audio generators together
pub struct AudioGraph {}

impl AudioGraph {
    // Create a new audio graph from an audio output
    pub fn new<T: Sample>(output: &impl AudioNode<T>) -> Self {
        todo!()
    }

    // Add a modifier to this audio graph
    pub fn modify<T: Sample>(self, modifier: &impl AudioModifier<T>) -> Self {
        self
    }

    // Add a mixer to this audio graph
    pub fn mix<T: Sample>(self, mixer: &impl AudioMixer<T>) -> Self {
        self
    }
}