use crate::{Source, SourceInput, Value};
use atomic_float::AtomicF32;
use std::{marker::PhantomData, sync::Arc, time::Duration};

pub struct Sine<V: Value>(V::Storage);
pub struct Square<V1: Value, V2: Value>(V1::Storage, V2::Storage);

impl<V: Value> Sine<V> {
    // Create a new sine waveform oscillator
    pub fn new(frequency: V) -> Self {
        Self(V::new_storage_from(frequency))
    }
}

impl<V1: Value, V2: Value> Square<V1, V2> {
    // Create a new square waveform oscillator
    pub fn new(frequency: V1, duty: V2) -> Self {
        Self(V1::new_storage_from(frequency), V2::new_storage_from(duty))
    }
}

impl<V: Value> Source for Sine<V> {
    fn cache(&mut self) {
        V::cache(&mut self.0)
    }

    fn sample(&mut self, input: &SourceInput) -> Option<f32> {
        let time = input.duration * V::fetch(&self.0);
        Some((time * 2.0 * std::f32::consts::PI).sin())
    }

    fn duration(&self) -> Option<Duration> {
        None
    }

    fn target_channels(&self) -> Option<u16> {
        None
    }

    fn target_sample_rate(&self) -> Option<u32> {
        None
    }
}

impl<V1: Value, V2: Value> Source for Square<V1, V2> {
    fn cache(&mut self) {
        V1::cache(&mut self.0);
        V2::cache(&mut self.1);
    }

    fn sample(&mut self, input: &SourceInput) -> Option<f32> {
        let time = input.duration * V1::fetch(&self.0);
        let duty = V2::fetch(&self.1) * 2.0;

        Some(if (time % 2.0) < duty { 1.0 } else { -1.0 })
    }

    fn duration(&self) -> Option<Duration> {
        None
    }

    fn target_channels(&self) -> Option<u16> {
        None
    }

    fn target_sample_rate(&self) -> Option<u32> {
        None
    }
}
