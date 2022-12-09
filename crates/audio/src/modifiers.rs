use std::{marker::PhantomData, time::Duration, sync::Arc};
use parking_lot::Mutex;

use crate::{traits::{AudioNode}, Sample};

// Volume modifier that will modify the volume of the audio source
pub struct Volume<IN: AudioNode> {
    pub(crate) input: IN,
    pub(crate) volume: f32,
}

impl<S: Sample, IN: AudioNode<S = S>> AudioNode for Volume<IN> {
    type S = S;
}

// Positional modifier that will take in a position and update the modifier based on the
// position of a player
pub struct Positional<IN: AudioNode> {
    pub(crate) input: IN,
    pub(crate) positional: Arc<Mutex<vek::Vec3<f32>>>,
}

impl<S: Sample, IN: AudioNode<S = S>> AudioNode for Positional<IN> {
    type S = S;
}

/*
// Reverb modifier that will add reverb to the audio source
pub struct ReverbModifier();

// Echo modifier that will add delay to the audio source
pub struct DelayModifier(pub Duration);
*/