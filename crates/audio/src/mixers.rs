use std::marker::PhantomData;
use crate::{traits::AudioNode, Sample};

// Simple mixer that uses n number of volume modifiers to mix the multiple tracks together
pub struct SimpleMixer<T: Sample, IN: AudioNode<T>> {
    _phantom: PhantomData<T>,
    input: IN,
}
