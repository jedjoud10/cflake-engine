use std::marker::PhantomData;
use crate::{traits::AudioNode, Sample};

// Volume modifier that will modify the volume of the audio source
pub struct VolumeModifier<T: Sample, IN: AudioNode<T>> {
    _phantom: PhantomData<T>,
    input: IN,
}

// Positional modifier that will take in a position and update the modifier based on the
// position of a player
pub struct PositionalModifier<T: Sample, IN: AudioNode<T>> {
    _phantom: PhantomData<T>,
    input: IN,
}

// Reverb modifier that will add reverb to the audio source
pub struct ReverbModifier<T: Sample, IN: AudioNode<T>> {
    _phantom: PhantomData<T>,
    input: IN,
}

// Echo modifier that will add delay to the audio source
pub struct DelayModifier<T: Sample, IN: AudioNode<T>> {
    _phantom: PhantomData<T>,
    input: IN,
}