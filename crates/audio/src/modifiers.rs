use std::marker::PhantomData;
use cpal::Sample;
use crate::traits::AudioOutput;

// Volume modifier that will modify the volume of the audio source
pub struct VolumeModifier<T: Sample, O: AudioOutput<T>> {
    _phantom: PhantomData<T>,
    input: O,
}

// Positional modifier that will take in a position and update the modifier based on the
// position of a player
pub struct PositionalModifier<T: Sample, O: AudioOutput<T>> {
    _phantom: PhantomData<T>,
    input: O,
}

// Reverb modifier that will add reverb to the audio source
pub struct ReverbModifier<T: Sample, O: AudioOutput<T>> {
    _phantom: PhantomData<T>,
    input: O,
}

// Echo modifier that will add delay to the audio source
pub struct DelayModifier<T: Sample, O: AudioOutput<T>> {
    _phantom: PhantomData<T>,
    input: O,
}